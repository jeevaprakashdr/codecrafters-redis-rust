use std::{arch::x86_64, collections::{HashMap, HashSet}, str::FromStr, sync::Arc};

use crate::redis::{command::Command, db::{self, DB, Value}, resp::create_bulk_string, stream::{Stream, StreamEntryId}};


pub struct XaddCommand<'a> {
    pub args: &'a [&'a str]
}

impl<'a> Command for XaddCommand<'a> {
    fn execute (&self) -> Result<String, &'static str> {
        let in_memory_db = Arc::clone(&DB);
        let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
        
        match db.get_mut(self.args[0].to_string()) {
            Some(data) => {
                let stream_entry_id = 
                    if self.args[1] == "*" {
                        StreamEntryId::default()
                    } else if self.args[1].contains("*") {
                        let new: StreamEntryId = StreamEntryId::from_str(self.args[1]).unwrap_or_default();
                        data.stream()
                            .iter()
                            .find(|x| x.id >= new)
                            .map(|x| StreamEntryId::new(x.id.ms, x.id.seqno + 1))
                            .unwrap_or(new)
                    } else {
                        StreamEntryId::from_str(self.args[1]).unwrap()
                    };
                println!("{}", stream_entry_id);

                if stream_entry_id == StreamEntryId::new(0,0) {
                    return Err("-ERR The ID specified in XADD must be greater than 0-0\r\n");
                }
                
                if data.stream().iter().any(|s| stream_entry_id <= s.id) {
                    return Err("-ERR The ID specified in XADD is equal or smaller than the target stream top item\r\n");
                }
                
                let stream_content = self.args
                    .iter()
                    .skip(2)
                    .map(|f| f.to_string())
                    .collect::<Vec<_>>();

                let mut en = HashMap::new();
                en.insert(stream_entry_id.to_string(), stream_content.to_owned());
                data.push_stream(Stream {
                    id: stream_entry_id,
                    entries: stream_content,
                });                
                Ok(create_bulk_string(stream_entry_id.to_string().as_str()))
            },
            None => {
                let stream_content = self.args
                    .iter()
                    .skip(2)
                    .map(|f| f.to_string())
                    .collect::<Vec<_>>();

                let key = self.args[0].to_string();
                let stream_entry_id = StreamEntryId::from_str(self.args[1]).unwrap();
                let mut en = HashMap::new();
                en.insert(stream_entry_id.to_string(), stream_content.to_owned());
                
                let value  = Value::with_stream(vec![Stream {
                    id: stream_entry_id,
                    entries: stream_content,
                }]);

                db.insert(key, value);

                Ok(create_bulk_string(stream_entry_id.to_string().as_str()))
            },
        }
    }
}