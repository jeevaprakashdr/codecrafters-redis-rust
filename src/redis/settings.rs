use std::process::Command;

pub struct RedisSetting {
    is_multi_mode: bool,
    pub command_queue: Vec<Command>,
}

impl RedisSetting {
    pub fn new() -> Self {
        Self { 
            is_multi_mode: false,
            command_queue: Vec::new()
        }
    }

    pub fn set_multi_mode(&mut self, multi_mode: bool) {
        self.is_multi_mode = multi_mode
    }

    pub fn get_multi_mode(&self) -> bool {
        self.is_multi_mode
    }
}