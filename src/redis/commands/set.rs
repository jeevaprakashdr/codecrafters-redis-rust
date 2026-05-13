use std::str::FromStr;
use std::sync::Arc;
use std::task::Context;

use chrono::Utc;

use crate::redis::commands::{Command, CommandHandlerContext, QueueContent, RedisCommand};
use crate::redis::db::{self, DB};
use crate::redis::resp::{self, create_simple_string};
use crate::redis::server::ServerContext;

pub struct Set<'a> {
    pub context: &'a mut CommandHandlerContext,
    pub args: Vec<String>,
}

impl<'a> Command for Set<'a> {
    fn execute(&mut self) -> Result<String, &'static str> {
        if self.context.is_multi_mode_on() {
            self.context.push(QueueContent {
                command: RedisCommand::Set,
                args: self.args.iter().map(|f| f.to_string()).collect::<Vec<String>>()
            });
            Ok(create_simple_string("QUEUED"))
        } else {
            execute_set(&self.args)
        }
    }
}

fn execute_set(args: &[String]) -> Result<String, &'static str> {
    let in_memory_db = Arc::clone(&DB);
    let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();

    if args.len() < 2 {
        return Err("Invalid arguments");
    }

    let key = args[0].clone();
    let val = args[1].to_string();

    let mut value = db::Value::with_str(val);
    if let Some(px_index) = args.iter().position(|s| s == "PX") {
        let milliseconds=  i64::from_str(args[px_index + 1].as_str()).unwrap_or_default();
        value.set_expire_at(Utc::now() + chrono::Duration::milliseconds(milliseconds));
    }
    db.insert(&key, value);
    Ok(resp::create_simple_string("OK"))
} 
