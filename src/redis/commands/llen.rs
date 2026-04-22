use std::sync::Arc;

use crate::redis::commands::Command;
use crate::redis::resp::create_simple_integer;
use crate::redis::db::{self, DB};

pub struct Llen<'a> {
    pub args: &'a [&'a str]
}

impl<'a> Command for Llen<'a> {
    fn execute (&self) -> Result<String, &'static str> {
       execute_llen(self.args)
    }
}

fn execute_llen(args: &[&str]) -> Result<String, &'static str> {
    let in_memory_db = Arc::clone(&DB);
    let db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();

    match db.get(args[0]) {
        Some(data) => {
            Ok(create_simple_integer(data.list().len()))
        }
        None => Ok(create_simple_integer(0))
    }
}