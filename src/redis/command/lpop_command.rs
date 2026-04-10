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

fn execute_lpop(command_array: &Vec<String>) -> Result<String, &'static str> {
    let number_of_elements = command_array
        .get(2)
        .map(|f |i32::from_str(f.as_str()).unwrap_or(0))
        .unwrap_or(0);

    pop(&command_array, number_of_elements)
}

fn pop(command_array: &Vec<String>, number_of_elements: i32) -> Result<String, &'static str> {
    let in_memory_db = Arc::clone(&DB);
    let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
    match db.get_mut(command_array[1].to_string()) {
        Some(data) => {
            if data.val.is_empty() {
                return Ok(create_null_bulk_string())
            }

            let current_val = std::mem::take(&mut data.val);
        
            if number_of_elements > 0 {
                let collection = current_val.split(",").collect::<Vec<_>>();
                let popped = collection[0 as usize..number_of_elements as usize].to_vec();
                data.val = collection[number_of_elements as usize..].join(",");
                return Ok(create_array(&popped))
            } else if number_of_elements > 0 && number_of_elements > i32::try_from(current_val.split(",").collect::<Vec<_>>().len()).unwrap_or(0) {
                let popped = std::mem::take(&mut data.val);
                return Ok(create_array(&popped.split(",").collect::<Vec<_>>()))
            }

            if let Some((first, rest)) = current_val.split_once(",") {
                let popped = first.to_string();
                data.val = rest.to_string();
                Ok(create_bulk_string(popped.as_str()))
            } else {
                let popped = std::mem::take(&mut data.val);
                data.val = "".to_string();
                Ok(create_bulk_string(popped.as_str()))
            } 
        }
        None => Ok(create_null_bulk_string())
    }
}