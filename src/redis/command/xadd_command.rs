use std::{arch::x86_64, collections::{HashMap, HashSet}, str::FromStr, sync::Arc};

use crate::redis::{command::Command, db::{self, DB, Value}, resp::create_bulk_string, stream::{Stream, StreamEntryId}};


pub struct XaddCommand {
    pub args: Vec<String>
}

impl Command for XaddCommand {
    fn execute (&self) -> Result<String, &'static str> {
        let in_memory_db = Arc::clone(&DB);
        let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
        
        match db.get_mut(self.args[1].to_string()) {
            Some(data) => {
                let stream_entry_id = 
                    if self.args[2] == "*" {
                        StreamEntryId::default()
                    } else if self.args[2].contains("*") {
                        let new: StreamEntryId = StreamEntryId::from_str(self.args[2].as_str()).unwrap_or_default();
                        data.stream
                            .iter()
                            .find(|x| x.id >= new)
                            .map(|x| StreamEntryId::new(x.id.ms, x.id.seqno + 1))
                            .unwrap_or(new)
                    } else {
                        StreamEntryId::from_str(self.args[2].as_str()).unwrap()
                    };
                println!("{}", stream_entry_id);

                if stream_entry_id == StreamEntryId::new(0,0) {
                    return Err("-ERR The ID specified in XADD must be greater than 0-0\r\n");
                }
                
                if data.stream.iter().any(|s| stream_entry_id == s.id || stream_entry_id < s.id) {
                    return Err("-ERR The ID specified in XADD is equal or smaller than the target stream top item\r\n");
                }
                
                let stream_content = self.args
                    .iter()
                    .skip(3)
                    .map(|s| s.clone())
                    .collect::<Vec<String>>();

                data.stream.push(Stream {
                    id: stream_entry_id,
                    entries: stream_content,
                });                
                Ok(create_bulk_string(stream_entry_id.to_string().as_str()))
            },
            None => {
                let stream_content = self.args
                    .iter()
                    .skip(3)
                    .map(|s| s.clone())
                    .collect::<Vec<String>>();

                let key = self.args[1].to_string();
                let stream_entry_id = StreamEntryId::from_str(self.args[2].as_str()).unwrap();
                let val  = Value {
                    val: "".to_string(),
                    data_type: None,
                    expire_at: None,
                    list: None, 
                    stream: vec![Stream {
                        id: stream_entry_id,
                        entries: stream_content,
                    }],
                };

                db.insert(key, val);
                Ok(create_bulk_string(stream_entry_id.to_string().as_str()))
            },
        }
        
        
    }
}