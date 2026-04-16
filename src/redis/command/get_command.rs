use std::sync::Arc;

use chrono::Utc;

use crate::redis::command::Command;
use crate::redis::resp;
use crate::redis::db::{self, DB};

pub struct GetCommand {
    pub args: Vec<String>
}

impl Command for GetCommand {
    fn execute (&self) -> Result<String, &'static str> {
       execute_get(&self.args)
    }
}

fn execute_get(command_array: &[String]) -> Result<String, &'static str> {
    let in_memory_db = Arc::clone(&DB);
    let db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
    match db.get(command_array[1].to_string()) {
        Some(v) if v.expire_at.map(|t| t <= Utc::now()).unwrap_or(false) => {
            Ok(resp::create_null_bulk_string())
        }
        Some(v) => {
            Ok(resp::create_bulk_string(v.val.as_str()))
        }
        None => {
            Ok(resp::create_null_bulk_string())
        },
    }
}