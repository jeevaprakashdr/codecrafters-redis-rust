use crate::redis::commands::Command;

pub struct InvalidCommand();

impl Command for InvalidCommand {
    fn execute(&mut self) -> Result<String, &'static str> {
        Err("INVALID COMMAND")
    }
}
