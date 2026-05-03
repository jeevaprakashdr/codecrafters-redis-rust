use std::sync::Arc;

use crate::redis::{commands::{self, Command, RedisCommand}, db::{self, DB}, resp::{create_array, create_array_bulk_string, create_empty_array, create_simple_string}, settings::RedisSetting};

pub struct Exec {
    pub redis_setting: Arc<std::sync::Mutex<RedisSetting>>,
}

impl Command for Exec {
    fn execute (&self) -> Result<String, &'static str> {
        if let setting = self.redis_setting.lock().unwrap() && !setting.get_multi_mode(){
            return Ok("-ERR EXEC without MULTI\r\n".to_string())
        }

        if let mut setting = self.redis_setting.lock().unwrap()
            && setting.get_multi_mode() 
            && setting.command_queue.is_empty() {
            setting.set_multi_mode(false);
            return  Ok(create_empty_array())
        }
        
        let queued_commands= {
            let mut setting = self.redis_setting.lock().unwrap();
            setting.set_multi_mode(false);
            
            let queues = setting.command_queue.drain(..).collect::<Vec<_>>();
            drop(setting);
            queues
        };

        let command_outputs: Vec<String> = queued_commands
            .iter()
            .map(|c| {
                let args: Vec<&str> = c.args
                    .iter()
                    .map(|arg| arg.as_str())
                    .collect();
                
                let command = RedisCommand::create_command(
                    Arc::clone(&self.redis_setting), 
                    c.command_str.as_str(), 
                    args.as_slice()
                );

                command.execute()

            })
            .filter(|result| result.is_ok())
            .map(|result| result.unwrap())
            .collect::<Vec<_>>();
        
        Ok(create_array(&command_outputs.iter().map(|r| r.as_str()).collect::<Vec<_>>().as_slice()))
    }
}