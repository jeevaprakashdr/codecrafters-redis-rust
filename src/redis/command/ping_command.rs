use crate::redis::resp;
use crate::redis::command::Command;

pub struct PingCommand();

impl Command for PingCommand {
    fn execute (&self) -> Result<String, &'static str> {
        Ok(resp::create_simple_string("PONG"))
    }
}