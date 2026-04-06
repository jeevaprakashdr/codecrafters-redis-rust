#![allow(unused_imports)]
use std::{collections::HashMap, io::{Read, Write}, net::TcpListener, ops::{Add, Index}, str, string, sync::{Arc, Mutex}, thread};

mod db;
mod resp;
mod redis;
use bytes::buf;
use chrono::Utc;

fn main() {

    let memory = Arc::new(Mutex::new(HashMap::new()));

    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    
    for stream in listener.incoming() {
        let memory = Arc::clone(&memory);
        thread::spawn(move || {
            redis::handle_connection(memory, stream);
        });
    }
}