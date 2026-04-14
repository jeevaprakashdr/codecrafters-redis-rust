use std::str::FromStr;
use std::sync::Arc;

use crate::redis::command::Command;
use crate::redis::db::{self, DB, Value};
use crate::redis::resp::create_bulk_string;
use crate::redis::stream::{StreamEntry, StreamEntryId};

pub struct XaddCommand {
    pub args: Vec<String>
}

impl Command for XaddCommand {
    fn execute (&self) -> Result<String, &'static str> {
        let in_memory_db = Arc::clone(&DB);
        let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
        
        let (stream_key, new_stream_entry_id, new_stream_entries) = parse(&self.args);

        match update_stream_last_entry_id(new_stream_entry_id.to_string(), &mut db) {
            Ok(()) => {}
            Err(EntryIdValidation::Default) => {
                return Err("-ERR The ID specified in XADD must be greater than 0-0\r\n");
            }
            Err(EntryIdValidation::EqualOrLess) => {
                return Err("-ERR The ID specified in XADD is equal or smaller than the target stream top item\r\n");
            }
        }

        match db.get_mut(stream_key.to_string()) {
            Some(current) => {
                let copied = std::mem::take(&mut current.val);
                current.val = format!(
                    "{},{}",
                    copied.as_str(),
                    new_stream_entries[1..].iter().map(|f| f.to_string()).collect::<Vec<_>>().join(","));
                Ok(create_bulk_string(new_stream_entry_id.as_str()))
            },
            None => {
                let value = Value {
                    val: new_stream_entries.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(","),
                    expire_at: None, 
                    data_type: Some("stream".to_string()),
                };
                db.insert(stream_key, value);
                db.insert("LSEID".to_string(), Value { val: new_stream_entry_id.to_string(), expire_at: None , data_type: None });
                Ok(create_bulk_string(new_stream_entry_id.as_str()))
            },
        }
    }
}


enum EntryIdValidation {
    Default,
    EqualOrLess,
}

fn update_stream_last_entry_id(new_stream_entry_id: String,  db: &mut db::InMemoryDb) 
    -> Result<(), EntryIdValidation> {
    if let Some(last_entry_id) = db.get_mut("LSEID".to_string()) {
        if new_stream_entry_id == "0-0"  {
            return Err(EntryIdValidation::Default);
        }

        if StreamEntryId::from_str(last_entry_id.val.as_str()).unwrap() 
            >= StreamEntryId::from_str(new_stream_entry_id.as_str()).unwrap() {
            return Err(EntryIdValidation::EqualOrLess);
        }
        
        last_entry_id.val = new_stream_entry_id.to_string();
    }
    
    Ok(())
}

fn parse(args: &Vec<String>) -> (String, String, Vec<StreamEntry>) {
    let (arg_key_val, _r) = args[1..].as_chunks::<2>();
    
    let mut entries = 
        vec![StreamEntry { key : "".to_string(), value: arg_key_val[0][1].to_string(),}];
    let mut remaining_entries: Vec<StreamEntry> = arg_key_val
        .iter()
        .map(|item| {
            StreamEntry { key: item[0].to_string(), value: item[1].to_string()}
        })
        .collect::<Vec<_>>();
    entries.append(&mut remaining_entries);      
          
    (arg_key_val[0][0].to_string(), arg_key_val[0][1].to_string(), entries)
}