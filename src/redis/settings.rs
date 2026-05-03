use std::process::Command;

#[derive(Debug)]
pub struct QueuedCommand {
    pub command_str: String,
    pub args: Vec<String>,
}

pub struct RedisSetting {
    is_multi_mode: bool,
    pub command_queue: Vec<QueuedCommand>,
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