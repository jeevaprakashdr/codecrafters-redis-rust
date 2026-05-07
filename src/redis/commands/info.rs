use std::sync::Arc;

use crate::redis::db::{self, DB};
use crate::redis::resp::{create_bulk_string, create_simple_string};
use crate::redis::commands::Command;

pub struct Info<'a> {
    pub args: &'a [&'a str],
}

impl<'a> Command for Info<'a> {
    fn execute (&self) -> Result<String, &'static str> {
        let in_memory_db = Arc::clone(&DB);
        let db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
        
        if self.args.contains(  &"replication") {
            let role = db.get("ROLE")
            .map(|val| val.str_val() )
            .unwrap_or("role:slave");
            
            return Ok(create_bulk_string(role));
        }
        
        Ok(create_simple_string("SERVER INFO NOT FOUND"))
    }
}
