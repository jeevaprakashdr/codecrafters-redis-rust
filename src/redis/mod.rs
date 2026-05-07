pub mod cli;
pub mod resp;
pub mod db;
pub mod commands;
pub mod settings;

mod stream;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use crate::redis::cli::Arguments;
use crate::redis::commands::RedisCommand;
use crate::redis::db::Value;
use crate::redis::settings::RedisSetting;

pub fn handle_connection(mut stream: TcpStream, arguments: Arguments) {
    println!("stream {:?}", stream);
    println!("accepted new connection");
    let redis_setting = Arc::new(Mutex::new(RedisSetting::new()));
    
    init_server_info(arguments);

    loop {
        let cmd = read_command_string(&mut stream);
        if let Ok(parsed_command_array) = resp::parse(cmd) {
            let response = RedisCommand::execute(Arc::clone(&redis_setting), &parsed_command_array)
                .inspect_err(|e| println!("{}", e))
                .unwrap_or_else(|e| e.to_string());
            stream.write_all(response.as_bytes()).unwrap();
        }
    }
}

fn init_server_info(arguments: Arguments) {
    let in_memory_db = Arc::clone(&db::DB);
    let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
    if arguments.replicaof.is_some() {
        db.insert("INFO", Value::with_str("role:slave".to_string()));
    } else {
        let info = vec![
                ReplicationInfo::with(ReplicationInfoKey::Role, "master".to_string()),
                ReplicationInfo::with(ReplicationInfoKey::MasterReplid, "8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb".to_string()),
                ReplicationInfo::with(ReplicationInfoKey::MasterReplOffset, "0".to_string())
            ]
            .iter()
            .map(|v|v.to_string())
            .collect::<Vec<_>>()
            .join("");
        db.insert("INFO", Value::with_str(info));
    }
}

fn read_command_string(stream: &mut TcpStream) -> String {
    let mut buffer = [0; 512];
    let  bytes_count = stream.read(&mut buffer[..]).unwrap();
    println!("buffer {:?}", str::from_utf8(&buffer[..bytes_count]));

    let cmd: &str = str::from_utf8(&buffer[..bytes_count]).unwrap();
    cmd.to_string()
}

#[derive(Debug)]
enum ReplicationInfoKey {
    Role,
    MasterReplid,
    MasterReplOffset,
}

impl std::fmt::Display for ReplicationInfoKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let key = match self {
            ReplicationInfoKey::Role => "role",
            ReplicationInfoKey::MasterReplid => "master_replid",
            ReplicationInfoKey::MasterReplOffset => "master_repl_offset",
        };
        write!(f, "{}", key)
    }
}

struct ReplicationInfo {
    key: ReplicationInfoKey,
    value: String
}

impl std::fmt::Display for ReplicationInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.key.to_string(), self.value)
    }
}

impl ReplicationInfo {
    fn with(key: ReplicationInfoKey, value: String) -> ReplicationInfo {
        ReplicationInfo {
            key,
            value
        }
    }
}