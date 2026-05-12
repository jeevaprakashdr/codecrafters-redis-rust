use std::{clone, collections::VecDeque, default, io::{self, Read}, net::{TcpListener, TcpStream}, sync::Arc, task::Context, thread};

use clap::Parser;

use crate::redis::{cli::ServerArguments, commands::{CommandHandler, CommandHandlerContext, RedisCommand}, resp, settings::QueuedCommand};

pub(crate) struct ServerContext {
    replication_info: ReplicationInfo,
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
                    command_handler.handle(&mut stream)
                }
            }
        });
    }
    
    fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
    
}

#[derive(Default, Clone)]
enum ReplicationRole {
    #[default]
    Master,
    Slave(String)
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
}

fn get_repl_info(args: &ServerArguments) -> ReplicationInfo {
    match &args.replicaof {
        Some(host) => {
            ReplicationInfo::new(
                ReplicationRole::Slave(host.to_string()), 
                Some("8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb".to_string()),
                Some(0),
            )
        }
        None => ReplicationInfo::new(ReplicationRole::Master,None,  None ),
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