use crate::redis::{commands::Command, resp::create_simple_string};

pub struct Multi<'a> {
    pub args: &'a [&'a str]
}

impl<'a> Command for Multi<'a> {
    fn execute (&self) -> Result<String, &'static str> {
        Ok(create_simple_string("OK"))
    }
}