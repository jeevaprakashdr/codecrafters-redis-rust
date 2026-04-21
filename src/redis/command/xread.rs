use std::{thread, time};
use std::time::Duration;
use std::{str::FromStr, sync::Arc};

use crate::redis::command::Command;
use crate::redis::stream::{Stream, StreamEntryId};
use crate::redis::db::{self, DB};
use crate::redis::resp::{create_array, create_bulk_string, create_empty_array, create_null_array};

pub struct Xread {
    pub args: Vec<String>,
}

impl Command for Xread {
    fn execute (&self) -> Result<String, &'static str> {
        let timeout =  self.get_timeout();
        let mut timeout_expired = false;
        let mut previous = String::new();

        loop {
            if let Ok(data) = self.execute() {
                if previous.is_empty() && self.on_block_get_latest_record(){
                    previous = data;
                } else if !previous.is_empty() && previous != data {
                    return Ok(data);
                } else if !self.on_block_get_latest_record() {
                    return Ok(data);
                }
            }
            
            if timeout_expired {
                return Ok(create_null_array())
            }

            if timeout > 0 {
                thread::sleep(time::Duration::from_millis(timeout as u64));
                timeout_expired = true;
            }
        }          
    }
    
}

impl Xread {
    fn has_blocking_request(&self) -> bool {
        self.args.iter().any(|arg| arg.to_lowercase() == "block".to_string())
    }
    
    fn get_timeout(&self) -> usize {
        if self.has_blocking_request() { usize::from_str(self.args[2].as_str()).unwrap_or(0) } else {  0 }
    }

    fn on_block_get_latest_record(&self) -> bool {
        self.args.last() == Some(&"$".to_string())
    }

    fn get_arg_stream_keys(&self) -> Vec<String> {
        let start_index = if self.has_blocking_request() { 4 } else { 2 };
        self.args
            .iter()
            .skip(start_index)
            .filter(|arg| !arg.contains("-"))
            .map(|arg| arg.to_string())
            .collect::<Vec<_>>()
    }

    fn get_arg_stream_entry_ids(&self) -> Vec<StreamEntryId> {
        let start_index = if self.has_blocking_request() { 4 } else { 2 };
        self.args
            .iter()
            .skip(start_index)
            .filter(|arg| arg.contains("-"))
            .map(|arg| StreamEntryId::from_str(arg.as_str())
                .unwrap_or(StreamEntryId { ms: 0, seqno: 0 }))
            .collect::<Vec<_>>()
    }

    fn execute(&self) -> Result<String, &'static str> {
        let stream_keys = self.get_arg_stream_keys();
        let start_entry_ids = self.get_arg_stream_entry_ids();
        
        let out: Vec<String> = stream_keys
            .iter()
            .enumerate()
            .map(|(index, key)|{
                if self.args.last() == Some(&"$".to_string()) {
                    fetch_latest_data(key)    
                } else {
                    fetch_data(*start_entry_ids.get(index).unwrap_or(&StreamEntryId { ms: 0, seqno: 0 }), key)
                }
            })
            .filter(|vec| !vec.is_empty())
            .map(|stream_data| format!("*{}\r\n{}", stream_data.len(), stream_data.join("")))
            .collect();

        if !out.is_empty() {
            Ok(format!("*{}\r\n{}", out.len(), out.join("")))
        } else {
            Err("No data found")
        }
    }
}

fn fetch_latest_data(stream_key: &String) -> Vec<String> {
    let in_memory_db = Arc::clone(&DB);
    let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
    
    match db.get_mut(stream_key.to_string()) {
        Some(data) => {
            let data: Vec<String> = data.stream.last().map_or_else(|| vec![], |stream| {
                let bs = create_bulk_string (stream.id.to_string().as_str());
                let arr = create_array(&stream.entries.iter().map(|val| val.as_str()).collect::<Vec<_>>());
                if arr.is_empty() { 
                    vec![] 
                } else { 
                    vec![bs, arr]
                }
            });
            
            let stream_data: Vec<String> = vec![data]
                .iter()
                .map(|streams|  format!("*{}\r\n{}", streams.len(), streams.join("")))
                .collect();

            let bs = create_bulk_string(stream_key.as_str());
            let stream_str = format!("*{}\r\n{}", stream_data.len(), stream_data.join(""));
            vec![bs, stream_str]
        }
        None => vec![]
    }
}

fn fetch_data(start: StreamEntryId, stream_key: &String) -> Vec<String> {
    let in_memory_db = Arc::clone(&DB);
    let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();

    match db.get_mut(stream_key.to_string()) {
        Some(data) => {

            let filtered: Vec<&Stream> = data.stream
                .iter()
                .filter(|stream| stream.id > start)
                .collect::<Vec<_>>();

            if filtered.is_empty() {
                return vec![];
            }

            let stream_data : Vec<String> = filtered
                .iter()
                .map(|stream| {
                    let bs = create_bulk_string (stream.id.to_string().as_str());
                    let arr = create_array(&stream.entries.iter().map(|val| val.as_str()).collect::<Vec<_>>());
                    if arr.is_empty() { vec![] } else { vec![bs, arr] }
                })
                .map(|streams|  format!("*{}\r\n{}", streams.len(), streams.join("")))
                .collect();                    
          
            let bs = create_bulk_string(stream_key.as_str());
            let stream_str = format!("*{}\r\n{}", stream_data.len(), stream_data.join(""));
            vec![bs, stream_str]            
        },
        None => { vec![] },
    }    
}

