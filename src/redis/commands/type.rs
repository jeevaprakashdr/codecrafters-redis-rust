use std::sync::Arc;

use crate::redis::{commands::Command, db::{self, DB}, resp::create_simple_string};

pub struct Type<'a> {
    pub args: &'a [&'a str]
}

impl<'a> Command for Type<'a> {
    fn execute (&self) -> Result<String, &'static str> {
       let in_memory_db = Arc::clone(&DB);
        let db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
        match db.get(self.args[0]) {
            Some(data) => {
                if !data.str_val().is_empty() {
                    return Ok(create_simple_string("string"))        
                }
                
                if !data.stream().is_empty() {
                    return Ok(create_simple_string("stream"))
                }

                Ok(create_simple_string("none"))
            },
            None => Ok(create_simple_string("none"))
        }
    }
}