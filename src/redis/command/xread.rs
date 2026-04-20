use std::{str::FromStr, sync::Arc};

use crate::redis::command::Command;
use crate::redis::stream::{Stream, StreamEntryId};
use crate::redis::db::{self, DB};
use crate::redis::resp::{create_array, create_bulk_string, create_empty_array};

pub struct Xread {
    pub args: Vec<String>
}

impl Command for Xread {
    fn execute (&self) -> Result<String, &'static str> {
        let in_memory_db = Arc::clone(&DB);
        let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();

        let default  = StreamEntryId { ms: 0, seqno: 0 };
        let start = StreamEntryId::from_str(self.args[3].as_str()).unwrap_or(default);
        let stream_keys = self.args.iter().skip(2).filter(|f| !f.contains("-")).collect::<Vec<_>>();

        let out: Vec<String> = stream_keys
            .iter()
            .map(|key|fetch_data(&mut db, start, key))
            .map(|stream_data| format!("*{}\r\n{}", stream_data.len(), stream_data.join("").to_string()))
            .collect();

        if out.len() > 0 {
            Ok(format!("*{}\r\n{}", out.len(), out.join("").to_string()))
        } else {
            Ok(create_empty_array())
        }
    }
}

fn fetch_data(db: &mut std::sync::MutexGuard<'_, db::InMemoryDb>, start: StreamEntryId, stream_key: &String) -> Vec<String> {
    match db.get_mut(stream_key.to_string()) {
        Some(data) => {

            let filtered: Vec<&Stream> = data.stream
                .iter()
                .filter(|stream| stream.id >= start)
                .collect::<Vec<_>>();

            let stream_data : Vec<String> = filtered
                .iter()
                .map(|stream| {
                    let mut streams = Vec::<String>::new();
                    streams.push(create_bulk_string (stream.id.to_string().as_str()));
                    streams.push(create_array(&stream.entries.iter().map(|val| val.as_str()).collect::<Vec<_>>()));
                    streams
                })
                .map(|streams|  format!("*{}\r\n{}", streams.len(), streams.join("")))
                .collect();                    
            
            let mut data = Vec::new();
            data.push(create_bulk_string(stream_key.as_str()));
            data.push(format!("*{}\r\n{}", stream_data.len(), stream_data.join("").to_string()));
            data
        },
        None => { vec![] },
    }    
}

