use std::sync::Arc;
use std::str::FromStr;

use crate::redis::{commands::Command, db::{self, DB}, resp::{create_empty_array, create_simple_integer}};

pub struct Incr<'a> {
    pub args: &'a [&'a str]
}

impl<'a> Command for Incr<'a> {
    fn execute (&self) -> Result<String, &'static str> {
        let in_memory_db = Arc::clone(&DB);
        let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
        
        match db.get_mut(self.args[0]) {
            Some(data) => {
                match usize::from_str(data.str_val()) {
                    Ok(int_val) => {
                        let new: usize = int_val + 1;
                        data.set_str_val(new.to_string().as_str());
                        Ok(create_simple_integer(new))
                    } ,
                    Err(_) => Err("Not int value"),
                }
            }
            None => Ok(create_empty_array()),
        }
    }
}

