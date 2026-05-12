use std::collections::VecDeque;
use std::thread;
use std::sync::Arc;
use std::net::{TcpListener, TcpStream};
use std::io::{self, Read};
use std::fmt::Display;
use clap::Parser;

use crate::redis::commands::{CommandHandler, CommandHandlerContext};
use crate::redis::resp;
use crate::redis::cli::ServerArguments;

pub(crate) struct ServerContext {
    replication_info: ReplicationInfo,
}

impl ServerContext {
    pub(crate) fn get_role_info(&self) -> Vec<String> {
        self.replication_info.get_role_info()
    }
}

pub(crate) struct Server<'a> {
    pub host: &'a str,
    pub port: u16,
    pub context: Arc<std::sync::Mutex<ServerContext>>
}

impl<'a> Server<'a> {
    pub(crate) fn new() -> Self {
        let arguments = ServerArguments::parse();
        let replication_info = get_repl_info(&arguments);

        Self {
            host: "127.0.0.1",
            port: arguments.port,
            context: Arc::new(std::sync::Mutex::new(ServerContext { 
                replication_info
            })),
        }
    }
    
    pub(crate) fn listener(&self) -> io::Result<TcpListener> {
        println!("Listening on {}:{}", self.host, self.port);
        TcpListener::bind((self.host, self.port))
    }

    pub(crate) fn handler_stream(&self, mut stream: TcpStream) {
        let mut context =  CommandHandlerContext::new(false, VecDeque::new());
        let server_context = self.context.clone();
        thread::spawn(move || {
            loop {
                let read_result = read_command_string(&mut stream);

                if read_result.is_err() {
                    break
                }

                let cmd_str= read_result.unwrap();
                if cmd_str.is_empty() {
                    break
                }

                if let Ok((cmd_str, args)) = resp::parse_with(cmd_str) {
                    let mut command_handler = CommandHandler::new(cmd_str, args, &mut context);
                    command_handler.handle(&stream, server_context.clone())
                }
            }
        });
    }
}

#[derive(Default, Clone, PartialEq)]
enum ReplicationRole {
    #[default]
    Master,
    Slave(String)
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

#[derive(Clone)]
struct ReplicationInfo {
    role: ReplicationRole,
    replication_id: Option<String>,
    replication_offset: Option<u16>
}

impl ReplicationInfo {
    fn new(role: ReplicationRole, replication_id: Option<String>, replication_offset: Option<u16>) -> Self {
        Self { role, replication_id, replication_offset }
    }
    
    fn get_role_info(&self) -> Vec<String> {
        let mut info:Vec<String> = Vec::new();
        info.push(self.role.to_string());
        info.push(format!("master_replid:{}", self.replication_id.clone().map_or("".to_string(), |f| f)));
        info.push(format!("master_repl_offset:{}", self.replication_offset.map_or(0, |f| f)));

        info
    }
}

fn get_repl_info(args: &ServerArguments) -> ReplicationInfo {
    match &args.replicaof {
        Some(host) => {
            ReplicationInfo::new(
                ReplicationRole::Slave(host.to_string()), 
                None,
                None,
            )
        }
        None => ReplicationInfo::new(
            ReplicationRole::Master,
            Some("8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb".to_string()),  
            Some(0)),
    }
}

fn read_command_string(stream: &mut TcpStream) -> Result<String, &'static str> {
    let mut buffer = [0; 512];

    let bytes_count = stream.read(&mut buffer[..])
    .map_err(|_| "failed to read data ")?;
    
    println!("buffer {:?}", str::from_utf8(&buffer[..bytes_count]));

    let cmd: &str = str::from_utf8(&buffer[..bytes_count]).unwrap();
    Ok(cmd.to_string())
}