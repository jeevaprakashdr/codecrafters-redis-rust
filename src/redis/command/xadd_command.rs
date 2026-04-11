use std::sync::Arc;

use crate::redis::command::Command;
use crate::redis::db::{self, DB, Value};
use crate::redis::resp::{create_bulk_string, create_null_bulk_string, create_simple_string};

pub struct XaddCommand {
    pub args: Vec<String>
}

impl Command for XaddCommand {
    fn execute (&self) -> Result<String, &'static str> {
        let in_memory_db = Arc::clone(&DB);
        let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
        match db.get(self.args[1].to_string()) {
            Some(_) => {
                Ok(create_null_bulk_string())
            },
            None => {
                let (key, id, val) = create(&self.args);
                db.insert(key, val);
                Ok(create_bulk_string(id.as_str()))
            },
        }
    }
}

fn create(args: &Vec<String>) -> (String, String, Value) {
    let (key_val, _r) = args[1..].as_chunks::<2>();
    let stream_key = &key_val[0][0];
    let stream_key_id = &key_val[0][1];
    
    let val = key_val
        .iter()
        .enumerate()
        .map(|(index, item)| {
            if index == 0 { 
                format!("ID:{}", item[1])
            } else {
                format!("{}:{}", item[0], item[1])
            }
        })
        .collect::<Vec<_>>()
        .join(",");
    
    (stream_key.to_string(), stream_key_id.to_string(), Value{ val, expire_at: None, data_type: Some("stream".to_string())})
}