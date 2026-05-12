use std::sync::Arc;

use crate::redis::server::ServerContext;
use crate::redis::resp::{create_array, create_empty_array};
use crate::redis::commands::{Command, CommandHandlerContext, QueueContent, RedisCommand};

pub struct Exec<'a> {
    pub server_context: Arc<std::sync::Mutex<ServerContext>>,
    pub context: &'a mut CommandHandlerContext,
}

impl<'a> Command for Exec<'a> {
    fn execute (&mut self) -> Result<String, &'static str> {
        println!("multi {}", self.context.is_multi_mode_on());
        if !self.context.is_multi_mode_on() {
            return Ok("-ERR EXEC without MULTI\r\n".to_string())
        }

        if self.context.is_multi_mode_on()
            && self.context.queue.is_empty() {
            self.context.set_multi_mode_off();
            return  Ok(create_empty_array())
        }
        
        self.context.set_multi_mode_off();
        let command_outputs: Vec<String> =  self.context.queue
            .drain(..)
            .collect::<Vec<_>>()
            .iter()
            .map(|content| {
                let mut command = RedisCommand::create(
                    content.command_str.clone(), 
                    content.args.clone(),
                    self.context,
                    self.server_context.clone()
                );

                command.execute()

            })
            .map(|result| {
                match result {
                    Ok(output) => output,
                    Err(err) => err.to_string(),
                }
            })
            .collect::<Vec<_>>();
        
        Ok(create_array(command_outputs.iter().map(|r| r.as_str()).collect::<Vec<_>>().as_slice()))
    }
}