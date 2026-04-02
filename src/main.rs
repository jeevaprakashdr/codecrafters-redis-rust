#![allow(unused_imports)]
use std::{collections::HashMap, io::{Read, Write}, net::TcpListener, ops::{Add, Index}, str, string, sync::{Arc, Mutex}, thread};

mod db;
mod resp;
use bytes::buf;
use chrono::Utc;

fn main() {

    let memory = Arc::new(Mutex::new(HashMap::new()));

    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    
    for stream in listener.incoming() {
        let memory = Arc::clone(&memory);
        thread::spawn(move || {
            handle_connection(memory, stream);
        });
    }
}

fn handle_connection(
    memory: Arc<Mutex<HashMap<String, db::Value>>>, 
    stream: Result<std::net::TcpStream, std::io::Error>) {
    match stream {
        Ok(mut stream) => {
            println!("accepted new connection");
            loop {
                let mut buffer = [0; 512];
                let  bytes_count = stream.read(&mut buffer[..]).unwrap();
                
                if bytes_count == 0 {
                    break;
                }
                
                let cmd = str::from_utf8(&buffer[..bytes_count]).unwrap();
                println!("{:?}", str::from_utf8(&buffer[..bytes_count]));
                if let Ok(parsed_collection) = resp::process(cmd) {
                    match parsed_collection[0].to_lowercase().as_str() {
                        "ping" => {
                            stream
                                .write(resp::create_simple_string("PONG").as_bytes())
                                .unwrap();
                        }
                        "echo" => {
                            stream
                                .write(
                                    resp::create_bulk_string(parsed_collection[1..].join(" ").as_str())
                                        .as_bytes())
                                .unwrap();
                        }
                         "set" => {
                            let mut db = memory.lock().unwrap();
                            
                            let mut value = db::Value { val: parsed_collection[2].to_string(), expire_at: None};
                            if let Some(index) = parsed_collection.iter().position(|s| s == "PX") {
                                let milliseconds: i64 =  parsed_collection[index+1].parse().unwrap();
                                value.expire_at = Some(Utc::now() + chrono::Duration::milliseconds(milliseconds));
                            }
                            db.insert(parsed_collection[1].to_string(), value);

                            stream
                                .write(resp::create_simple_string("OK").as_bytes())
                                .unwrap();
                         }
                        "get" => {
                            let db: std::sync::MutexGuard<'_, HashMap<String, db::Value>> = memory.lock().unwrap();
                            match db.get(&parsed_collection[1]) {
                                Some(v) if v.expire_at.map(|t| t <= Utc::now()).unwrap_or(false) => {
                                    let _ = stream.write(resp::create_null_bulk_string().as_bytes());
                                }
                                Some(v) => {
                                    stream
                                        .write(resp::create_bulk_string(v.val.as_str()).as_bytes())
                                        .unwrap();
                                }
                                None => {
                                    let _ = stream.write(resp::create_null_bulk_string().as_bytes());
                                },
                            }
                        }
                        _ => {
                            println!("Failed to process command")
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("error: {}", e);
        }
    }
}
