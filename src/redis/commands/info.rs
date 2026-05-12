use std::sync::Arc;

use crate::redis::db::{self, DB};
use crate::redis::resp::{create_bulk_string, create_simple_string};
use crate::redis::commands::Command;

pub struct Info {
    pub args: Vec<String>,
}

impl Command for Info {
    fn execute (&mut self) -> Result<String, &'static str> {
        let in_memory_db = Arc::clone(&DB);
        let db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
        
        if self.args.contains(&"replication".to_string()) {
            let role = db.get("INFO")
            .map(|val| val.str_val() )
            .unwrap_or("role:slave");
            
            return Ok(create_bulk_string(role));
        }
        
        Ok(create_simple_string("SERVER INFO NOT FOUND"))
    }
}
