use std::{fmt::Display, str::FromStr, sync::Arc};

use chrono::Utc;

use crate::redis::resp;
use crate::redis::db::{self, DB};


#[derive(Debug, PartialEq)]
pub enum Command {
    Ping, 
    Echo,
    Set,
    Get,
    Lpush,
    Rpush,
    Lrange,
}

impl Command {
    pub fn execute(
        command_array: Vec<String>) -> Result<String, &'static str> {
        match Command::from_str(command_array[0].as_str()) {
            Ok(Command::Ping) => Ok(resp::create_simple_string("PONG")),
            Ok(Command::Echo) => Ok(resp::create_bulk_string(command_array[1..].join(" ").as_str())),
            Ok(Command::Set) => execute_set(command_array),
            Ok(Command::Get) => execute_get(command_array),
            Ok(Command::Lpush) => execute_lpush(command_array),
            Ok(Command::Rpush) => execute_rpush(command_array),
            Ok(Command::Lrange) => execute_lrange(command_array),
            _ => {
                Err("Failed to process command")
            }
        }
    }
}
fn execute_lrange(command_array: Vec<String>) -> Result<String, &'static str> {
    let in_memory_db_clone = Arc::clone(&DB);
    let db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db_clone.lock().unwrap();

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

        return Ok(resp::create_array(&collection[start_index as usize ..=stop_index as usize]))
    }
    else {
        return Ok(resp::create_empty_array())
    }    
}

fn execute_lpush(command_array: Vec<String>) -> Result<String, &'static str> {
    let in_memory_db_clone = Arc::clone(&DB);
    let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db_clone.lock().unwrap();
    match db.get_mut(command_array[1].to_string()) {
        Some(record) => {
            let args = command_array[2..]
            .iter()
            .rev()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(",");

            record.val = format!("{},{}", args, record.val);
            Ok(resp::create_simple_integer(
                i32::try_from(record.val.split(',').count())
                    .unwrap_or(1)))
        },
        None => {
            let val = command_array[2..]
                .iter()
                .rev()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(",");
            let value = db::Value { val, expire_at: None};
            db.insert(command_array[1].to_string(), value);
            Ok(resp::create_simple_integer(i32::try_from(command_array[2..].iter().count()).unwrap_or(1)))
        }
    }
}

fn execute_rpush(command_array: Vec<String>) -> Result<String, &'static str> {
    let in_memory_db_clone = Arc::clone(&DB);
    let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db_clone.lock().unwrap();
    match db.get_mut(command_array[1].to_string()) {
        Some(record) => {
            record.val = format!("{},{}", record.val, command_array[2..].join(","));
            Ok(resp::create_simple_integer(
                i32::try_from(record.val.split(',').count())
                    .unwrap_or(1)))
        },
        None => {
            let value = db::Value { val: command_array[2..].join(","), expire_at: None};
            db.insert(command_array[1].to_string(), value);
            Ok(resp::create_simple_integer(i32::try_from(command_array[2..].iter().count()).unwrap_or(1)))
        }
    }
}

fn execute_get(command_array: Vec<String>) -> Result<String, &'static str> {
    let in_memory_db_clone = Arc::clone(&DB);
    let db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db_clone.lock().unwrap();
    match db.get(command_array[1].to_string()) {
        Some(v) if v.expire_at.map(|t| t <= Utc::now()).unwrap_or(false) => {
            Ok(resp::create_null_bulk_string())
        }
        Some(v) => {
            Ok(resp::create_bulk_string(v.val.as_str()))
        }
        None => {
            Ok(resp::create_null_bulk_string())
        },
    }
}

fn execute_set(command_array: Vec<String>) -> Result<String, &'static str> {
    let in_memory_db_clone = Arc::clone(&DB);
    let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db_clone.lock().unwrap();
    let mut value = db::Value { val: command_array[2].to_string(), expire_at: None};
    if let Some(index) = command_array.iter().position(|s| s == "PX") {
        let milliseconds: i64 =  command_array[index+1].parse().unwrap();
        value.expire_at = Some(Utc::now() + chrono::Duration::milliseconds(milliseconds));
    }
    db.insert(command_array[1].to_string(), value);
    Ok(resp::create_simple_string("OK"))
}


impl FromStr for Command {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ping" => Ok(Command::Ping),
            "echo" => Ok(Command::Echo),
            "set" => Ok(Command::Set),
            "get" => Ok(Command::Get),
            "lpush" => Ok(Command::Lpush),
            "rpush" => Ok(Command::Rpush),
            "lrange" => Ok(Command::Lrange),
            _ => Err(format!("unknown command: {}", s))
        }
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Ping => write!(f, "ping"),
            Command::Echo => write!(f, "echo"),
            Command::Set => write!(f, "set"),
            Command::Get => write!(f, "get"),
            Command::Lpush => write!(f, "lpush"),
            Command::Rpush => write!(f, "rpush"),
            Command::Lrange => write!(f, "lrange"),
        }
    }
}

