use std::sync::Arc;

use crate::redis::commands::Command;
use crate::redis::resp::create_simple_integer;
use crate::redis::db::{self, DB};

pub struct Llen {
    pub args: Vec<String>
}

impl<'a> Command for Llen {
    fn execute (&mut self) -> Result<String, &'static str> {
       execute_llen(&self.args)
    }
}

fn execute_llen(args: &Vec<String>) -> Result<String, &'static str> {
    let in_memory_db = Arc::clone(&DB);
    let db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();

    match db.get(args[0].as_str()) {
        Some(data) => {
            Ok(create_simple_integer(data.list().len()))
        }
        None => Ok(create_simple_integer(0))
    }
}