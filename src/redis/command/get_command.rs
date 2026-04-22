use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::redis::command::Command;
use crate::redis::resp;
use crate::redis::db::{self, DB};

pub struct GetCommand<'a> {
    pub args: &'a [&'a str]
}

impl<'a> Command for GetCommand<'a> {
    fn execute (&self) -> Result<String, &'static str> {
       execute_get(self.args)
    }
}

fn execute_get(args: &[&str]) -> Result<String, &'static str> {
    let in_memory_db = Arc::clone(&DB);
    let db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
    match db.get(args[0].to_string()) {
        Some(value) if value.expire_at().map(is_expired).unwrap_or_default() => {
            Ok(resp::create_null_bulk_string())
        }
        Some(value) => {
            Ok(resp::create_bulk_string(value.str_val()))
        }
        None => {
            Ok(resp::create_null_bulk_string())
        },
    }
}

fn is_expired(t: DateTime<Utc>) -> bool {
    t < Utc::now()
}