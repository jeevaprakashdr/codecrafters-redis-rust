use std::sync::Arc;

use chrono::Utc;

use crate::redis::resp;
use crate::redis::db::{self, DB};
use crate::redis::commands::Command;

pub struct Lpush<'a> {
    pub args: &'a [&'a str]
}

impl<'a> Command for Lpush<'a> {
    fn execute (&self) -> Result<String, &'static str> {
       execute_lpush(self.args)
    }
}

fn execute_lpush(args: &[&str]) -> Result<String, &'static str> {
    let in_memory_db = Arc::clone(&DB);
    let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
    match db.get_mut(args[0]) {
        Some(data) => {
            let mut current= data.list().to_vec();
            let mut new = args[1..].iter().map(|f| f.to_string()).collect();
            current.append(&mut new);
            current.reverse();
            
            data.set_list(&current);
            Ok(resp::create_simple_integer(current.len()))
        },
        None => {
            let list_items: Vec<String> = args[1..]
                .iter()
                .rev()
                .map(|f| f.to_string())
                .collect();
            let count = list_items.len();
            let value = db::Value::with_list(list_items);
            db.insert(args[0], value);
            Ok(resp::create_simple_integer(count))
        }
    }
}