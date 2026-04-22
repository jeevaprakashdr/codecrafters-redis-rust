use crate::redis::resp;
use crate::redis::commands::Command;

pub struct Ping();

impl Command for Ping {
    fn execute (&self) -> Result<String, &'static str> {
        Ok(resp::create_simple_string("PONG"))
    }
}