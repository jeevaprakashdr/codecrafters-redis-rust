use std::sync::Arc;

use crate::redis::command::Command;
use crate::redis::resp::create_simple_integer;
use crate::redis::db::{self, DB};

pub struct LlenCommand {
    pub args: Vec<String>
}

impl Command for LlenCommand {
    fn execute (&self) -> Result<String, &'static str> {
       execute_llen(&self.args)
    }
}

fn execute_llen(args: &Vec<String>) -> Result<String, &'static str> {
    let in_memory_db = Arc::clone(&DB);
    let db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();

    match db.get(args[1].to_string()) {
        Some(data) => {
            let len = data.list.as_ref().map(|f| f.len()).unwrap_or(0);
            Ok(create_simple_integer(len))
        }
        None => Ok(create_simple_integer(0))
    }
}