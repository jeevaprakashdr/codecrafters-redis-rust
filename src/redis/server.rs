use clap::Parser;
use std::collections::VecDeque;
use std::fmt::Display;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str::FromStr;
use std::sync::Arc;
use std::thread;

use crate::redis::cli::ServerArguments;
use crate::redis::commands::{CommandHandler, CommandHandlerContext, RedisCommand};
use crate::redis::resp::{self, create_array_bulk_string, create_bulk_string, create_resp_array};

pub(crate) struct ServerContext {
    replication_info: ReplicationInfo,
}

impl ServerContext {
    pub(self) fn new(replication_info: ReplicationInfo) -> Self {
        Self { replication_info }
    }

    pub(crate) fn get_role_info(&self) -> Vec<String> {
        self.replication_info.get_role_info()
    }
    
    pub(crate) fn get_replication_id(&self) -> String {
        self.replication_info.get_replication_id()
    }

    pub(crate) fn is_replica(&self) -> bool {
        !self.replication_info.is_master()
    }
}

pub(crate) struct Server<'a> {
    pub host: &'a str,
    pub port: u16,
    pub master_host: Option<String>,
    pub context: Arc<std::sync::Mutex<ServerContext>>,
}

impl<'a> Server<'a> {
    pub(crate) fn new() -> Self {
        let arguments = ServerArguments::parse();
        let replication_info = get_repl_info(&arguments);
        let master_host = arguments
            .replicaof
            .map(|master_host_info| master_host_info.replace(" ", ":"));

        Self {
            host: "127.0.0.1",
            port: arguments.port,
            master_host,
            context: Arc::new(std::sync::Mutex::new(ServerContext::new(replication_info))),
        }
    }

    pub(crate) fn listener(&self) -> io::Result<TcpListener> {
        println!("Listening on {}:{}", self.host, self.port);
        TcpListener::bind((self.host, self.port))
    }

    pub(crate) fn handler_stream(&self, mut stream: TcpStream) {
        let mut context = CommandHandlerContext::new(false, VecDeque::new());
        let server_context = self.context.clone();
        thread::spawn(move || {
            loop {
                let read_result = read_stream(&mut stream);
                if read_result.is_err() {
                    break;
                }

                let cmd_str = read_result.unwrap();
                if cmd_str.is_empty() {
                    break;
                }

                if let Ok(command) = resp::parse(cmd_str) {
                    let mut command_handler = CommandHandler::new(command, &mut context);
                    command_handler.handle(&stream, server_context.clone())
                }
            }
        });
    }

    pub(crate) fn is_replica(&self) -> bool {
        self.context.lock().unwrap().is_replica()
    }

    pub(crate) fn handshake_with_master(&self) {
        if self.master_host.is_none() {
            eprintln!("NO `master` host address found!!!");
            return;
        }

        let addr = self.master_host.as_ref().unwrap();
        match TcpStream::connect(addr) {
            Ok(mut stream) => {
                println!("Establishing handshake with master {:?}", stream);
                let command = create_array_bulk_string(&[&RedisCommand::Ping.to_string()]);
                send_command(&mut stream, command);
                let command = create_array_bulk_string(&["REPLCONF", "listening-port", "6380"]);
                send_command(&mut stream, command);
                let command = create_array_bulk_string(&["REPLCONF", "capa", "psync2"]);
                send_command(&mut stream, command);
                let command = create_array_bulk_string(&["PSYNC", "?", "-1"]);
                send_command(&mut stream, command);
            }
            Err(e) => eprintln!("error: {}", e),
        }
    }
}

fn send_command(stream: &mut TcpStream, command: String) {
    stream.write_all(command.as_bytes()).unwrap();
    let result = read_stream(stream);
    if let Err(e) = result {
        eprintln!("Failed establish handshake with master.\nCommand:{}\nError:{}", command, e);
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
enum ReplicationRole {
    #[default]
    Master,
    Slave(String),
}

impl Display for ReplicationRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let role = match self {
            ReplicationRole::Master => "master",
            ReplicationRole::Slave(_) => "slave",
        };

        write!(f, "role:{}", role)
    }
}

#[derive(Clone, Debug)]
struct ReplicationInfo {
    role: ReplicationRole,
    replication_id: Option<String>,
    replication_offset: Option<u16>,
}

impl ReplicationInfo {
    fn new(
        role: ReplicationRole,
        replication_id: Option<String>,
        replication_offset: Option<u16>,
    ) -> Self {
        Self {
            role,
            replication_id,
            replication_offset,
        }
    }

    fn get_role_info(&self) -> Vec<String> {
        let mut info: Vec<String> = Vec::new();
        info.push(self.role.to_string());
        info.push(format!(
            "master_replid:{}",
            self.replication_id.clone().map_or("".to_string(), |f| f)
        ));
        info.push(format!(
            "master_repl_offset:{}",
            self.replication_offset.map_or(0, |f| f)
        ));

        info
    }

    fn is_master(&self) -> bool {
        self.role == ReplicationRole::Master
            && self.replication_offset.is_some()
            && self.replication_id.is_some()
    }
    
    fn get_replication_id(&self) -> String {
        self.replication_id.clone().map_or("".to_string(), |f| f)
    }
}

fn get_repl_info(args: &ServerArguments) -> ReplicationInfo {
    match &args.replicaof {
        Some(host) => ReplicationInfo::new(ReplicationRole::Slave(host.to_string()), None, None),
        None => ReplicationInfo::new(
            ReplicationRole::Master,
            Some("8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb".to_string()),
            Some(0),
        ),
    }
}

fn read_stream(stream: &mut TcpStream) -> Result<String, &'static str> {
    let mut buffer = [0; 512];

    let bytes_count = stream
        .read(&mut buffer[..])
        .map_err(|_| "failed to read data ")?;

    println!("buffer {:?}", str::from_utf8(&buffer[..bytes_count]));

    let cmd: &str = str::from_utf8(&buffer[..bytes_count]).unwrap();
    Ok(cmd.to_string())
}
