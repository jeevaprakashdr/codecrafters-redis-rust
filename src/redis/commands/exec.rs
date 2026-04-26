use std::sync::Arc;

use crate::redis::{commands::Command, db::{self, DB}, resp::{create_empty_array, create_simple_string}, settings::RedisSetting};

pub struct Exec {
    pub redis_setting: Arc<std::sync::Mutex<RedisSetting>>,
}

impl Command for Exec {
    fn execute (&self) -> Result<String, &'static str> {
        let mut setting = self.redis_setting.lock().unwrap();
        
        if setting.get_multi_mode() && setting.command_queue.is_empty(){
            setting.set_multi_mode(false);
            Ok(create_empty_array())
        } else if setting.get_multi_mode() && !setting.command_queue.is_empty(){
            Ok(create_simple_string("Ok"))
        } else {
            Ok("-ERR EXEC without MULTI\r\n".to_string())
        }
    }
}