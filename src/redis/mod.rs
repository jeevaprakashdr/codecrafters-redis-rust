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
    println!("Saving server information -- begin");
    let in_memory_db = Arc::clone(&db::DB);
    let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
    if arguments.replicaof.is_some() {
        db.insert("ROLE", Value::with_str("role:slave".to_string()));
    } else {
        db.insert("ROLE", Value::with_str("role:master".to_string()));
    }
    println!("Saving server information -- completed");
}

fn read_command_string(stream: &mut TcpStream) -> String {
    let mut buffer = [0; 512];
    let  bytes_count = stream.read(&mut buffer[..]).unwrap();
    println!("buffer {:?}", str::from_utf8(&buffer[..bytes_count]));

    let cmd: &str = str::from_utf8(&buffer[..bytes_count]).unwrap();
    cmd.to_string()
}