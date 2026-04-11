use std::sync::Arc;

use chrono::Utc;

use crate::redis::resp;
use crate::redis::db::{self, DB};
use crate::redis::command::Command;

pub struct RpushCommand {
    pub args: Vec<String>
}

impl Command for RpushCommand {
    fn execute (&self) -> Result<String, &'static str> {
       execute_rpush(&self.args)
    }
}

fn execute_rpush(command_array: &Vec<String>) -> Result<String, &'static str> {
    let in_memory_db = Arc::clone(&DB);
    let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
    match db.get_mut(command_array[1].to_string()) {
        Some(record) => {
            if record.val.is_empty() {
                record.val =  command_array[2..].join(",");
            } else {
                record.val = format!("{},{}", record.val, command_array[2..].join(","));
            }
            
            Ok(resp::create_simple_integer(
                i32::try_from(record.val.split(',').count())
                    .unwrap_or(1)))
        },
        None => {
            let value = db::Value { val: command_array[2..].join(","), expire_at: None, data_type: None};
            db.insert(command_array[1].to_string(), value);
            Ok(resp::create_simple_integer(i32::try_from(command_array[2..].iter().count()).unwrap_or(1)))
        }
    }
}