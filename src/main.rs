#![allow(unused_imports)]
mod redis;

use std::{net::TcpListener, sync::Arc, thread};

use crate::redis::db::DB;
use crate::redis::handle_connection;

fn main() {
    let _ = Arc::clone(&DB);
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(tcp_stream) => {
                thread::spawn(move || {
                    handle_connection(tcp_stream);
                });
            },
            Err(e) => println!("error: {}", e),
        }
    }
}