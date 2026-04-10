use std::sync::Arc;

use chrono::Utc;

use crate::redis::resp;
use crate::redis::db::{self, DB};
use crate::redis::command::Command;

pub struct LpushCommand {
    pub args: Vec<String>
}

impl Command for LpushCommand {
    fn execute (&self) -> Result<String, &'static str> {
       execute_lpush(&self.args)
    }
}

fn execute_lpush(command_array: &Vec<String>) -> Result<String, &'static str> {
    let in_memory_db = Arc::clone(&DB);
    let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
    match db.get_mut(command_array[1].to_string()) {
        Some(record) => {
            let args = command_array[2..]
            .iter()
            .rev()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(",");

            record.val = format!("{},{}", args, record.val);
            Ok(resp::create_simple_integer(
                i32::try_from(record.val.split(',').count())
                    .unwrap_or(1)))
        },
        None => {
            let val = command_array[2..]
                .iter()
                .rev()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(",");
            let value = db::Value { val, expire_at: None};
            db.insert(command_array[1].to_string(), value);
            Ok(resp::create_simple_integer(i32::try_from(command_array[2..].iter().count()).unwrap_or(1)))
        }
    }
}