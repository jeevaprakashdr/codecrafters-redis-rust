use std::sync::Arc;

use crate::redis::{commands::Command, db::{self, DB}, resp::{create_empty_array, create_simple_string}};

pub struct Exec {
    pub client_id: u64,
}

impl Command for Exec {
    fn execute (&self) -> Result<String, &'static str> {
        let in_memory_db = Arc::clone(&DB);
        let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
        
        match db.get_mut(self.client_id.to_string().as_str()) {
            Some(data) => {
                if data.list().is_empty() {
                    Ok("-ERR EXEC without MULTI\r\n".to_string())
                } else if data.list()[1..].is_empty() {
                    data.set_list(&[]);
                    Ok(create_empty_array())
                } else {
                    Ok(create_empty_array())
                }
            },
            None => Ok("-ERR EXEC without MULTI\r\n".to_string()),
        }
        
    }
}