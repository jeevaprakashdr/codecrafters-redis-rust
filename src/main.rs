#![allow(unused_imports)]
mod redis;

use std::{collections::HashMap, io::{Read, Write}, net::TcpListener, ops::{Add, Index}, str, string, sync::{Arc, Mutex}, thread};
use bytes::buf;
use chrono::Utc;

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