use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::redis::commands::{Command, CommandHandlerContext, QueueContent, RedisCommand};
use crate::redis::db::{self, DB};
use crate::redis::resp::{self, create_simple_string};
use crate::redis::server::ServerContext;

pub struct Get<'a> {
    pub context: &'a mut CommandHandlerContext,
    pub args: Vec<String>,
}

impl<'a> Command for Get<'a> {
    fn execute(&mut self) -> Result<String, &'static str> {
        if self.context.is_multi_mode_on() {
            self.context.push(QueueContent {
                command: RedisCommand::Get,
                args: self
                    .args
                    .iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<String>>(),
            });
            return Ok(create_simple_string("QUEUED"));
        }

        execute_get(&self.args)
    }
}

fn execute_get(args: &[String]) -> Result<String, &'static str> {
    let in_memory_db = Arc::clone(&DB);
    let db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
    match db.get(args[0].as_str()) {
        Some(data) if data.expire_at().map(is_expired).unwrap_or_default() => {
            Ok(resp::create_null_bulk_string())
        }
        Some(data) => Ok(resp::create_bulk_string(data.str_val())),
        None => Ok(resp::create_null_bulk_string()),
    }
}

fn is_expired(t: DateTime<Utc>) -> bool {
    t < Utc::now()
}
