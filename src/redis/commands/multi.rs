use std::sync::Arc;

use crate::redis::settings::RedisSetting;
use crate::redis::resp::create_simple_string;
use crate::redis::commands::Command;

pub struct Multi {
    pub redis_setting: Arc<std::sync::Mutex<RedisSetting>>,
}

impl Command for Multi {
    fn execute (&self) -> Result<String, &'static str> {
        let mut setting = self.redis_setting.lock().unwrap();
        setting.set_multi_mode(true);
        
        Ok(create_simple_string("OK"))
    }
}