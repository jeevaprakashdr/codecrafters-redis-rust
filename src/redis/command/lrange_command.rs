use std::sync::Arc;

use std::str::FromStr;
use chrono::Utc;

use crate::redis::resp;
use crate::redis::db::{self, DB};
use crate::redis::command::Command;

pub struct LrangeCommand {
    pub args: Vec<String>
}

impl Command for LrangeCommand {
    fn execute (&self) -> Result<String, &'static str> {
       execute_lrange(&self.args)
    }
}

fn execute_lrange(command_array: &Vec<String>) -> Result<String, &'static str> {
    let in_memory_db = Arc::clone(&DB);
    let db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();

    if let Some(v) = db.get(command_array[1].to_string()) {
        let collection: Vec<&str> = v.val.split(',').collect();
        let collection_len = collection.len().try_into().unwrap_or(0);
        let start_index = i32::from_str(command_array[2].as_str())
            .map(|s| if s < 0 { (s + collection_len).max(0) } else { s })
            .unwrap_or(0);
        let mut stop_index = i32::from_str(command_array[3].as_str())
            .map(|s| if s < 0 { s + collection_len } else { s })
            .unwrap_or(0);

        // If the start index is greater than the stop index, an empty array is returned.
        // If the start index is greater than or equal to the list's length, an empty array is returned.
        if start_index > stop_index
            || start_index >= collection_len {
            return Ok(resp::create_empty_array())
        }
        
        // If the stop index is greater than or equal to the list's length, the stop index is treated as the last element.
        if stop_index >= collection_len {
            stop_index = collection.len().try_into().unwrap_or(1) - 1; // could be confusing 
            return Ok(resp::create_array(&collection[start_index as usize ..=stop_index as usize]))
        }
        println!("startIndex {}", start_index);
        println!("stopIndex {}", stop_index);
        return Ok(resp::create_array(&collection[start_index as usize ..=stop_index as usize]))
    }
    else {
        return Ok(resp::create_empty_array())
    }    
}