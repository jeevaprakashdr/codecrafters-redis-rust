use std::sync::Arc;
use std::str::FromStr;

use crate::redis::db::{self, DB};
use crate::redis::server::ServerContext;
use crate::redis::resp::{create_simple_integer, create_simple_string};
use crate::redis::commands::{Command, CommandHandlerContext, QueueContent, RedisCommand};

pub struct Incr<'a> {
    pub context: &'a mut CommandHandlerContext,
    pub args: Vec<String>,
}

impl<'a> Command for Incr<'a> {
    fn execute (&mut self) -> Result<String, &'static str> {
        if self.context.is_multi_mode_on() {
            let args = self.args.iter().map(|f| f.to_string()).collect::<Vec<String>>();
            self.context.push(QueueContent { command: RedisCommand::Incr(args) });
            return Ok(create_simple_string("QUEUED"))
        }

        let in_memory_db = Arc::clone(&DB);
        let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
        
        match db.get_mut(self.args[0].as_str()) {
            Some(data) => {
                match usize::from_str(data.str_val()) {
                    Ok(int_val) => {
                        let new: usize = int_val + 1;
                        data.set_str_val(new.to_string().as_str());
                        Ok(create_simple_integer(new))
                    } ,
                    Err(_) => Err("-ERR value is not an integer or out of range\r\n"),
                }
            }
            None => {
                db.insert(self.args[0].as_str(), db::Value::with_str("1".to_string()));
                Ok(create_simple_integer(1))
            },
        }
    }
}

