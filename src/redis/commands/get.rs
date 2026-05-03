use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::redis::commands::Command;
use crate::redis::resp::{self, create_simple_string};
use crate::redis::db::{self, DB};
use crate::redis::settings::{QueuedCommand, RedisSetting};

pub struct Get<'a> {
    pub args: &'a [&'a str],
    pub redis_setting: Arc<std::sync::Mutex<RedisSetting>>
}

impl<'a> Command for Get<'a> {
    fn execute (&self) -> Result<String, &'static str> {
        let mut setting = self.redis_setting.lock().unwrap();
        if setting.get_multi_mode() {
            let command = QueuedCommand {
                command_str: "GET".to_string(),
                args: self.args.iter().map(|f| f.to_string()).collect::<Vec<String>>()
            };
            
            setting.command_queue.push(command);
            return Ok(create_simple_string("QUEUED"))
        }
        
        execute_get(self.args)
    }
}

fn execute_get(args: &[&str]) -> Result<String, &'static str> {
    let in_memory_db = Arc::clone(&DB);
    let db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
    match db.get(args[0]) {
        Some(data) if data.expire_at().map(is_expired).unwrap_or_default() => {
            Ok(resp::create_null_bulk_string())
        }
        Some(data) => {
            Ok(resp::create_bulk_string(data.str_val()))
        }
        None => {
            Ok(resp::create_null_bulk_string())
        },
    }
}

fn is_expired(t: DateTime<Utc>) -> bool {
    t < Utc::now()
}