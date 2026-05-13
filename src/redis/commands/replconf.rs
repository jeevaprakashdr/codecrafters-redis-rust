use std::env::args;

use crate::redis::{commands::Command, resp::{create_null_bulk_string, create_simple_string}};

pub struct Replconf{
    pub args: Vec<String>
}

impl Command for Replconf {
    fn execute(&mut self) -> Result<String, &'static str> {
        if self.args.contains(&"listening-port".to_string()) {
            Ok(create_simple_string("OK"))
        } else if self.args.contains(&"capa".to_string()) {
            Ok(create_simple_string("OK"))
        } else {
            Ok(create_null_bulk_string())
        }
    }
}