use std::sync::Arc;

use crate::redis::{command::Command, db::{self, DB}, resp::create_simple_string};

pub struct TypeCommand {
    pub args: Vec<String>
}

impl Command for TypeCommand {
    fn execute (&self) -> Result<String, &'static str> {
       let in_memory_db = Arc::clone(&DB);
        let db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
        match db.get(self.args[1].to_string()) {
            Some(_) => Ok(create_simple_string("string")),
            None => Ok(create_simple_string("none"))
        }
    }
}