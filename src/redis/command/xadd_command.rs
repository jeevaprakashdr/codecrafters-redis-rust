use std::{collections::HashMap, str::FromStr, sync::Arc};

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
                let stream = data.stream.as_mut().unwrap();

                let stream_entry_id = 
                    if self.args[2] == "*" {
                        StreamEntryId::default()
                    } else if self.args[2].contains("*") {
                        let new = StreamEntryId::from_str(self.args[2].as_str()).unwrap_or_default();
                        let ms = if new.ms > stream.id.ms { new.ms } else { stream.id.ms };
                        let seqno = if new.ms >= stream.id.ms && new.seqno >= stream.id.seqno { new.seqno + 1 } else { new.seqno };
                        StreamEntryId::new(ms, seqno)
                    } else {
                        StreamEntryId::from_str(self.args[2].as_str()).unwrap()
                    };
                println!("{}", stream_entry_id);

                if stream_entry_id == StreamEntryId::new(0,0) {
                    return Err("-ERR The ID specified in XADD must be greater than 0-0\r\n");
                }
                
                if stream.id >= stream_entry_id {
                    return Err("-ERR The ID specified in XADD is equal or smaller than the target stream top item\r\n");
                }
                
                let (arg_key_val, _r) = self.args.as_chunks::<2>();
                for ele in arg_key_val.iter().skip(1) {
                    stream.entries.insert(ele[0].to_owned(), ele[1].to_owned());
                }
                stream.id = stream_entry_id;
                
                Ok(create_bulk_string(stream_entry_id.to_string().as_str()))
            },
            None => {
                let (arg_key_val, _r) = self.args.as_chunks::<2>();
                let mut entries : HashMap<String, String> = HashMap::new();
                for ele in arg_key_val.iter().skip(1) {
                    entries.insert(ele[0].to_owned(), ele[1].to_owned());
                }

                let key = self.args[1].to_string();
                let stream_entry_id = StreamEntryId::from_str(self.args[2].as_str()).unwrap();
                let val  = Value {
                    val: "".to_string(),
                    data_type: None,
                    expire_at: None,
                    list: None, 
                    stream: Some(Stream {
                        id: stream_entry_id,
                        entries,
                    }),
                };

                db.insert(key, val);
                Ok(create_bulk_string(stream_entry_id.to_string().as_str()))
            },
        }
        
        
    }
}