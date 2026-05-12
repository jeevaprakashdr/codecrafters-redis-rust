use crate::redis::resp;
use crate::redis::commands::Command;

pub struct Ping();

impl Command for Ping {
    fn execute (&mut self) -> Result<String, &'static str> {
        Ok(resp::create_simple_string("PONG"))
    }
}