use std::sync::Arc;

use chrono::Utc;

use crate::redis::resp;
use crate::redis::db::{self, DB};
use crate::redis::command::Command;
use crate::redis::stream::Stream;

pub struct LpushCommand {
    pub args: Vec<String>
}

impl Command for LpushCommand {
    fn execute (&self) -> Result<String, &'static str> {
       execute_lpush(&self.args)
    }
}

fn execute_lpush(args: &[String]) -> Result<String, &'static str> {
    let in_memory_db = Arc::clone(&DB);
    let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
    match db.get_mut(args[1].to_string()) {
        Some(record) => {
            let mut current= record.list.as_mut().map_or(Vec::new(), |v| v.to_vec());
            let mut update = args[2..].iter().map(|f| f.to_string()).collect();
            current.append(&mut update);
            current.reverse();
            let count = current.len();
            
            record.list = Some(current);
            Ok(resp::create_simple_integer(count))
        },
        None => {
            let list_items: Vec<String> = args[2..]
                .iter()
                .rev()
                .map(|f| f.to_string())
                .collect();
            let count = list_items.len();
            let value = db::Value {
                val: "".to_string(),
                expire_at: None, 
                data_type: None,
                list: Some(list_items),
                stream: vec![]
            };
            db.insert(args[1].to_string(), value);
            Ok(resp::create_simple_integer(count))
        }
    }
}