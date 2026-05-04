pub mod resp;
pub mod db;
pub mod commands;
pub mod settings;

mod stream;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use crate::redis::commands::RedisCommand;
use crate::redis::settings::RedisSetting;

pub fn handle_connection(mut stream: TcpStream) {
    println!("stream {:?}", stream);
    println!("accepted new connection");
    let redis_setting = Arc::new(Mutex::new(RedisSetting::new()));
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

fn read_command_string(stream: &mut TcpStream) -> String {
    let mut buffer = [0; 512];
    let  bytes_count = stream.read(&mut buffer[..]).unwrap();
    println!("buffer {:?}", str::from_utf8(&buffer[..bytes_count]));

    let cmd: &str = str::from_utf8(&buffer[..bytes_count]).unwrap();
    cmd.to_string()
}