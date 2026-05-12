#![allow(unused_imports)]
mod redis;

use std::{net::TcpListener, sync::Arc, thread};
use clap::Parser;

use crate::redis::db::DB;
use crate::redis::cli::ServerArguments;
use crate::redis::server::Server;

fn main() {
    let _ = Arc::clone(&DB);
    println!("Logs from your program will appear here!");

    let server = Server::new();
    let listener = server.listener().unwrap();

    if server.is_replica() {
        server.replica_handshake();
    }
    
    for stream in listener.incoming() {
        match stream {
            Ok(tcp_stream) => {
                println!("stream {:?}", tcp_stream);
                server.handler_stream(tcp_stream);
            },
            Err(e) => eprintln!("error: {}", e),
        }
    }
}