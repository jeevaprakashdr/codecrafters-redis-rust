use crate::redis::{commands::Command, resp::create_simple_string};

pub struct Exec {
}

impl Command for Exec {
    fn execute (&self) -> Result<String, &'static str> {
        Ok("-ERR EXEC without MULTI\r\n".to_string())
    }
}