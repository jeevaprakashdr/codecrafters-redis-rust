#![allow(unused_imports)]
mod redis;

use std::{net::TcpListener, sync::Arc, thread};
use clap::Parser;

use crate::redis::db::DB;
use crate::redis::handle_connection;
use crate::redis::cli::Arguments;

fn main() {
    let arguments = Arguments::parse();

    let _ = Arc::clone(&DB);
    println!("Logs from your program will appear here!");

    let addr = format!("127.0.0.1:{}", arguments.port);
    let listener = TcpListener::bind(addr).unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(tcp_stream) => {
                let arguments = arguments.clone();
                thread::spawn(move || {
                    handle_connection(tcp_stream, arguments);
                });
            },
            Err(e) => println!("error: {}", e),
        }
    }
}