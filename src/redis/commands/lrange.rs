use std::sync::Arc;

use std::str::FromStr;
use chrono::Utc;

use crate::redis::resp;
use crate::redis::db::{self, DB};
use crate::redis::commands::Command;

pub struct Lrange<'a> {
    pub args: &'a [&'a str]
}

impl<'a> Command for Lrange<'a> {
    fn execute (&self) -> Result<String, &'static str> {
       execute_lrange(self.args)
    }
}

fn execute_lrange(args: &[&str]) -> Result<String, &'static str> {
    let in_memory_db = Arc::clone(&DB);
    let db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();

    if let Some(data) = db.get(args[0]) {
        let len = data.list().len() as isize;

         let start_index = isize::from_str(args[1])
            .map(|s| normalize_index(len, s))
            .unwrap_or(0)
            .clamp(0, len);

        let stop_index = isize::from_str(args[2])
            .map(|s| normalize_index(len, s))
            .unwrap_or(0)
            .clamp(-1, len - 1);
            
         
        // If the start index is greater than the stop index, an empty array is returned.
        // If the start index is greater than or equal to the list's length, an empty array is returned.
        if start_index > stop_index || start_index >= len {
            return Ok(resp::create_empty_array())
        }

        let result = data.list()[start_index as usize..=stop_index as usize]
            .iter()
            .map(|ele| ele.as_str())
            .collect::<Vec<_>>();

        return Ok(resp::create_array(&result))
    }
    
    Ok(resp::create_empty_array())
}

fn normalize_index(len: isize, index: isize) -> isize {
    if index < 0 { index + len } else { index }
}