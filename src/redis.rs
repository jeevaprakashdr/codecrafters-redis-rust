use std::collections::{self, HashMap};
use std::fmt::Display;
use std::io::{Read, Write};
use std::str::FromStr;
use std::{
    io::Error, 
    net::TcpStream};
use std::sync::{Arc, Mutex};
use chrono::Utc;

use crate::{db, resp};

#[derive(Debug, PartialEq)]
pub enum Command {
    Ping, 
    Echo,
    Set,
    Get
}

impl FromStr for Command {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ping" => Ok(Command::Ping),
            "echo" => Ok(Command::Echo),
            "set" => Ok(Command::Set),
            "get" => Ok(Command::Get),
            _ => Err(format!("unknown command: {}", s))
        }
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Ping => write!(f, "ping"),
            Command::Echo => write!(f, "ping"),
            Command::Set => write!(f, "ping"),
            Command::Get => write!(f, "ping"),
        }
    }
}

pub fn handle_connection(
    memory: Arc<Mutex<HashMap<String, db::Value>>>, 
    stream: Result<TcpStream, Error>) {
    match stream {
        Ok(mut stream) => {
            println!("accepted new connection");
            loop {
                let mut buffer = [0; 512];
                let  bytes_count = stream.read(&mut buffer[..]).unwrap();
                
                if bytes_count == 0 {
                    break;
                }
                
                println!("{:?}", str::from_utf8(&buffer[..bytes_count]));

                let cmd = str::from_utf8(&buffer[..bytes_count]).unwrap();
                if let Ok(parsed_command_array) = resp::parse(cmd) {
                    match execute_command(&memory, parsed_command_array) {
                        Ok(response) => {
                            stream.write(response.as_bytes()).unwrap();
                        },
                        Err(e) => {
                            println!("{}", e);
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

fn execute_command(
    memory: &Arc<Mutex<HashMap<String, db::Value>>>,
    command_array: Vec<String>) -> Result<String, &'static str> {
    match Command::from_str(command_array[0].as_str()) {
        Ok(Command::Ping) => {
            Ok(resp::create_simple_string("PONG"))
        }
        Ok(Command::Echo) => {
            Ok(resp::create_bulk_string(command_array[1..].join(" ").as_str()))
        }
        Ok(Command::Set) => {
            let mut db = memory.lock().unwrap();
        
            let mut value = db::Value { val: command_array[2].to_string(), expire_at: None};
            if let Some(index) = command_array.iter().position(|s| s == "PX") {
                let milliseconds: i64 =  command_array[index+1].parse().unwrap();
                value.expire_at = Some(Utc::now() + chrono::Duration::milliseconds(milliseconds));
            }
            db.insert(command_array[1].to_string(), value);

            Ok(resp::create_simple_string("OK"))
         }
        Ok(Command::Get) => {
            let db: std::sync::MutexGuard<'_, HashMap<String, db::Value>> = memory.lock().unwrap();
            match db.get(&command_array[1]) {
                Some(v) if v.expire_at.map(|t| t <= Utc::now()).unwrap_or(false) => {
                    Ok(resp::create_null_bulk_string())
                }
                Some(v) => {
                    Ok(resp::create_bulk_string(v.val.as_str()))
                }
                None => {
                    Ok(resp::create_null_bulk_string())
                },
            }
        }
        _ => {
            Err("Failed to process command")
        }
    }
}