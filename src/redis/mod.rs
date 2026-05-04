pub mod resp;
pub mod db;
pub mod commands;
pub mod settings;

mod stream;

use std::collections::{self, HashMap};
use std::fmt::Display;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{Read, Write};
use std::os::fd::IntoRawFd;
use std::str::FromStr;
use std::{
    io::Error, 
    net::TcpStream};
use std::sync::{Arc, Mutex};
use chrono::Utc;

use crate::redis::commands::RedisCommand;
use crate::redis::settings::RedisSetting;

pub fn handle_connection(mut stream: TcpStream) {
    println!("stream {:?}", stream);
    println!("accepted new connection");
    let redis_setting = Arc::new(Mutex::new(RedisSetting::new()));
    loop {
        let mut buffer = [0; 512];
        let  bytes_count = stream.read(&mut buffer[..]).unwrap();
        
        if bytes_count == 0 {
            break;
        }
        
        println!("buffer {:?}", str::from_utf8(&buffer[..bytes_count]));

        let cmd = str::from_utf8(&buffer[..bytes_count]).unwrap();
        if let Ok(parsed_command_array) = resp::parse(cmd) {
            let command_array = &parsed_command_array.iter().map(|f| f.as_str()).collect::<Vec<_>>();
            match RedisCommand::execute(Arc::clone(&redis_setting), command_array) {
                Ok(response) => {
                    stream.write_all(response.as_bytes()).unwrap();
                },
                Err(e) => {
                    println!("{}", e);
                    stream.write_all(e.as_bytes()).unwrap();
                },
            }
        }
    }
}