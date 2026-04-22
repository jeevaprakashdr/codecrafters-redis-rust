use std::sync::Arc;

use chrono::Utc;

use crate::redis::resp;
use crate::redis::db::{self, DB};
use crate::redis::command::Command;
use crate::redis::stream::Stream;

pub struct RpushCommand {
    pub args: Vec<String>
}

impl Command for RpushCommand {
    fn execute (&self) -> Result<String, &'static str> {
       execute_rpush(&self.args)
    }
}

fn execute_rpush(args: &[String]) -> Result<String, &'static str> {
    let in_memory_db = Arc::clone(&DB);
    let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
    match db.get_mut(args[1].to_string()) {
        Some(record) => {
            let mut current= record.list().to_vec();
            let mut update = list_items(args);
            current.append(&mut update);
            
            record.set_list(&current);
            Ok(resp::create_simple_integer(current.len()))
        },
        None => {
            let list_items = list_items(args);
            let count = list_items.len();
            let value = db::Value::with_list(list_items);
            db.insert(args[1].to_string(), value);
            Ok(resp::create_simple_integer(count))
        }
    }
}

fn list_items(args: &[String]) -> Vec<String> {
    args[2..].iter().map(|f| f.to_string()).collect()
}