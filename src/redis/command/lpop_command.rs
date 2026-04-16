use std::sync::Arc;

use std::str::FromStr;

use crate::redis::resp::{create_array, create_bulk_string, create_null_bulk_string};
use crate::redis::db::{self, DB};
use crate::redis::command::Command;
pub struct LpopCommand {
    pub args: Vec<String>
}

impl Command for LpopCommand {
    fn execute (&self) -> Result<String, &'static str> {
       execute_lpop(&self.args)
    }
}

fn execute_lpop(args: &[String]) -> Result<String, &'static str> {
    let in_memory_db = Arc::clone(&DB);
    let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
    match db.get_mut(args[1].to_string()) {
        Some(data) => {
            if data.list.iter().len() == 0 {
                return Ok(create_null_bulk_string())
            }

            let current= data.list.as_mut().map_or(Vec::new(), |v| v.to_vec());
            let number_of_elements = args.get(2)
                .map(|f |usize::from_str(f.as_str()).unwrap_or(0))
                .unwrap_or(1);
         
            data.list = Some(current[number_of_elements..].to_vec());
            
            let popped = current[0..number_of_elements].to_vec();
            if popped.len() == 1 {
                Ok(create_bulk_string(popped[0].as_str()))
            } else {
                Ok(create_array(&popped.iter().map(|s|s.as_str()).collect::<Vec<_>>()))
            }
        }
        None => Ok(create_null_bulk_string())
    }
}