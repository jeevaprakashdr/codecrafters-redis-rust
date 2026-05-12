use crate::redis::commands::Command;
use crate::redis::resp;

pub struct Echo {
    pub args: Vec<String>
}

impl Command for Echo {
    fn execute (&mut self) -> Result<String, &'static str> {
        Ok(resp::create_bulk_string(self.args.join(" ").as_str()))
    }
}