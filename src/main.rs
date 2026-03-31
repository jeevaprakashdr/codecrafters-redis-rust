#![allow(unused_imports)]
use std::{io::{Read, Write}, net::TcpListener, thread};

use bytes::buf;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    
    for stream in listener.incoming() {
        thread::spawn(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(stream: Result<std::net::TcpStream, std::io::Error>) {
    match stream {
        Ok(mut stream) => {
            println!("accepted new connection");
            loop {
                let mut buffer = [0; 512];
                let  bytes_count = stream.read(&mut buffer[..]).unwrap();
                
                if bytes_count == 0 {
                    break;
                }

                stream.write(b"+PONG\r\n").unwrap();
            }
        }
        Err(e) => {
            println!("error: {}", e);
        }
    }
}
