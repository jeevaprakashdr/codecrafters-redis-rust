use crate::redis::command::Command;
use crate::redis::resp;

pub struct EchoCommand {
    pub args: Vec<String>
}

impl Command for EchoCommand {
    fn execute (&self) -> Result<String, &'static str> {
        Ok(resp::create_bulk_string(self.args[1..].join(" ").as_str()))
    }
}