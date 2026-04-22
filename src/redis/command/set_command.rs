use std::sync::Arc;

use chrono::Utc;

use crate::redis::resp;
use crate::redis::db::{self, DB};
use crate::redis::command::Command;
use crate::redis::stream::Stream;

pub struct SetCommand<'a> {
    pub args: &'a [&'a str]
}

impl<'a> Command for SetCommand<'a> {
    fn execute (&self) -> Result<String, &'static str> {
       execute_set(&self.args)
    }
}

fn execute_set(args: &[&str]) -> Result<String, &'static str> {
    let in_memory_db = Arc::clone(&DB);
    let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
    
    let key = args[0].to_string();
    let val = args[1].to_string();
    
    let mut value = db::Value::with_str(val);
    if let Some(index) = args.iter().position(|s| s == &"PX") {
        let milliseconds: i64 =  args[index+1].parse().unwrap();
        value.set_expire_at(Utc::now() + chrono::Duration::milliseconds(milliseconds));
    }
    db.insert(key, value);
    Ok(resp::create_simple_string("OK"))
}