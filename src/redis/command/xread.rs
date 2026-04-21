use std::{thread, time};
use std::time::Duration;
use std::{str::FromStr, sync::Arc};

use crate::redis::command::Command;
use crate::redis::stream::{Stream, StreamEntryId};
use crate::redis::db::{self, DB};
use crate::redis::resp::{create_array, create_bulk_string, create_empty_array, create_null_array};

pub struct Xread {
    pub args: Vec<String>
}

impl Command for Xread {
    fn execute (&self) -> Result<String, &'static str> {
        let blocking_request = self.args.iter().any(|arg| arg.to_lowercase() == "block".to_string());
        let (args_skip_index, timeout) = 
            if blocking_request { (4, u64::from_str(self.args[2].as_str()).unwrap_or(0)) } else { (2, 0) };
        
        let mut timeout_expired = false;
        loop {
            if let Ok(data) = self.execute(args_skip_index) {
                return Ok(data)
            }
            
            if timeout_expired {
                println!("expired");
                return Ok(create_null_array())
            }

            if timeout > 0 {
                println!("blocked");
                thread::sleep(time::Duration::from_secs(1));
                timeout_expired = true
            }
        }    
    }
}

impl Xread {
    fn execute(&self, args_skip_index: usize) -> Result<String, &'static str> {
        let in_memory_db = Arc::clone(&DB);
        let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
        
        let default  = StreamEntryId { ms: 0, seqno: 0 };
        let stream_keys = self.args.iter().skip(args_skip_index).filter(|f| !f.contains("-")).collect::<Vec<_>>();
        let start_entry_ids = self.args
            .iter()
            .skip(args_skip_index)
            .filter(|f| f.contains("-")).map(|f| StreamEntryId::from_str(f.as_str()).unwrap_or(default))
            .collect::<Vec<_>>();
        println!("stream_keys {:?} start_entry_ids {:?}", stream_keys, start_entry_ids);
        let out: Vec<String> = stream_keys
            .iter()
            .enumerate()
            .map(|(index, key)|fetch_data(&mut db, start_entry_ids[index], key))
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

fn fetch_data(db: &mut std::sync::MutexGuard<'_, db::InMemoryDb>, start: StreamEntryId, stream_key: &String) -> Vec<String> {
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

