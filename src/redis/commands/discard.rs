use std::sync::Arc;

use crate::redis::settings::RedisSetting;
use crate::redis::resp::create_simple_string;
use crate::redis::commands::Command;

pub struct Discard {
    pub redis_setting: Arc<std::sync::Mutex<RedisSetting>>,
}

impl Command for Discard {
    fn execute (&self) -> Result<String, &'static str> {
        if let setting = self.redis_setting.lock().unwrap() && !setting.get_multi_mode(){
            return Ok("-ERR DISCARD without MULTI\r\n".to_string())
        }

        let mut setting = self.redis_setting.lock().unwrap();
        setting.command_queue.clear();
        setting.set_multi_mode(false);

        Ok(create_simple_string("OK"))
    }
}

