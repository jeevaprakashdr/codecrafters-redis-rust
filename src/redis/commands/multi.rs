use std::sync::Arc;

use crate::redis::{commands::Command, db::{self, DB, Value}, resp::create_simple_string};

pub struct Multi {
    pub client_id: u64,
}

impl Command for Multi {
    fn execute (&self) -> Result<String, &'static str> {
        let in_memory_db = Arc::clone(&DB);
        let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
        
        match db.get_mut(self.client_id.to_string().as_str()) {
            Some(_) => {},
            None => {
                let val = Value::with_list(vec!["MULTI".to_string()]);
                db.insert(self.client_id.to_string().as_str(), val);
            },
        }
        
        Ok(create_simple_string("OK"))
    }
}