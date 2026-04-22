use crate::redis::commands::Command;
use crate::redis::resp;

pub struct Echo<'a> {
    pub args: &'a [&'a str]
}

impl<'a> Command for Echo<'a> {
    fn execute (&self) -> Result<String, &'static str> {
        Ok(resp::create_bulk_string(self.args.join(" ").as_str()))
    }
}