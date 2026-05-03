use std::sync::Arc;

use std::str::FromStr;

use crate::redis::resp::{create_array_bulk_string, create_bulk_string, create_null_bulk_string};
use crate::redis::db::{self, DB};
use crate::redis::commands::Command;
pub struct Lpop<'a> {
    pub args: &'a [&'a str]
}

impl<'a> Command for Lpop<'a> {
    fn execute (&self) -> Result<String, &'static str> {
       execute_lpop(self.args)
    }
}

fn execute_lpop(args: &[&str]) -> Result<String, &'static str> {
    let in_memory_db = Arc::clone(&DB);
    let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
    match db.get_mut(args[0]) {
        Some(data) => {
            if data.list().is_empty() {
                return Ok(create_null_bulk_string())
            }

            let current= data.list().to_vec();
            let number_of_elements = args.get(1)
                .map(|f |usize::from_str(f).unwrap_or(0))
                .unwrap_or(1);
         
            data.set_list(&current[number_of_elements..]);
            
            if current[0..number_of_elements].len() == 1 {
                Ok(create_bulk_string(current[0..number_of_elements].first().unwrap().as_str()))
            } else {
                Ok(create_array_bulk_string(&current[0..number_of_elements].iter().map(|s|s.as_str()).collect::<Vec<_>>()))
            }
        }
        None => Ok(create_null_bulk_string())
    }
}