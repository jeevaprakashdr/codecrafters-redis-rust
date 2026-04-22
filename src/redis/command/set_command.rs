use std::sync::Arc;

use chrono::Utc;

use crate::redis::resp;
use crate::redis::db::{self, DB};
use crate::redis::command::Command;
use crate::redis::stream::Stream;

pub struct SetCommand {
    pub args: Vec<String>
}

impl Command for SetCommand {
    fn execute (&self) -> Result<String, &'static str> {
       execute_set(&self.args)
    }
}

fn execute_set(command_array: &[String]) -> Result<String, &'static str> {
    let in_memory_db = Arc::clone(&DB);
    let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
    let mut value = db::Value { val: command_array[2].to_string(), expire_at: None, list:None, stream: vec![]};
    if let Some(index) = command_array.iter().position(|s| s == "PX") {
        let milliseconds: i64 =  command_array[index+1].parse().unwrap();
        value.expire_at = Some(Utc::now() + chrono::Duration::milliseconds(milliseconds));
    }
    db.insert(command_array[1].to_string(), value);
    Ok(resp::create_simple_string("OK"))
}