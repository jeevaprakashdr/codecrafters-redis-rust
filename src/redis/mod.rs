pub mod resp;
pub mod db;
pub mod commands;

mod stream;

use std::collections::{self, HashMap};
use std::fmt::Display;
use std::io::{Read, Write};
use std::str::FromStr;
use std::{
    io::Error, 
    net::TcpStream};
use std::sync::{Arc, Mutex};
use chrono::Utc;

use crate::redis::commands::RedisCommand;

pub fn handle_connection(
    stream: Result<TcpStream, Error>) {
    match stream {
        Ok(mut stream) => {
            println!("stream {:?}", stream);
            println!("accepted new connection");
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
                    match RedisCommand::execute(command_array) {
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
        Err(e) => {
            println!("error: {}", e);
        }
    }
}