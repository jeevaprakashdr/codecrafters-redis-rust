use crate::redis::command::Command;
use crate::redis::resp;

pub struct EchoCommand<'a> {
    pub args: &'a [&'a str]
}

impl<'a> Command for EchoCommand<'a> {
    fn execute (&self) -> Result<String, &'static str> {
        Ok(resp::create_bulk_string(self.args.join(" ").as_str()))
    }
}