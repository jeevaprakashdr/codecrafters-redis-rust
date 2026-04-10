use crate::redis::{command::Command, db::{self, DB}, resp::{create_array, create_null_array}};
use std::{fmt::Display, str::FromStr, sync::Arc, thread, time::Duration};

pub struct BlPopCommand {
    pub args: Vec<String>
}

impl Command for BlPopCommand {
    fn execute (&self) -> Result<String, &'static str> {
       execute_blpop(&self.args)
    }
}

fn execute_blpop(command_array: &Vec<String>) -> Result<String, &'static str> { 
    let timeout = command_array
                .get(2)
                .map(|f |f64::from_str(f.as_str()).unwrap_or(0.0))
                .unwrap_or(0.0);
    
    let mut timeout_expired = false;

    loop {
        if let Some(popped) = blpop(&command_array) {
            return Ok(create_array(&[command_array[1].as_str(), popped.as_str()]))
        }

        if timeout_expired {
            return Ok(create_null_array())
        }

        if timeout > 0.0 {
            thread::sleep(Duration::from_secs_f64(timeout));
            timeout_expired = true;
        }
    }    
}

fn blpop(command_array: &Vec<String>) -> Option<String> {
    let in_memory_db = Arc::clone(&DB);
    let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
    match db.get_mut(command_array[1].to_string()) {
        Some(data) => {
            if data.val.is_empty() {
                return None
            } 
            
            if let Some((first, rest)) = data.val.split_once(",") {
                let popped = first.to_string();
                data.val = rest.to_string();
                Some(popped)
            } else {
                let popped = std::mem::take(&mut data.val);
                Some(popped)
            } 
        }
        None => None
    }
}