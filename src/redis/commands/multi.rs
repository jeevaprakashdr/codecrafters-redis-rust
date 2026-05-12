use std::sync::Arc;

use crate::redis::server::ServerContext;
use crate::redis::resp::create_simple_string;
use crate::redis::commands::{Command, CommandHandlerContext};

pub struct Multi<'a> {
    pub context: &'a mut CommandHandlerContext,
}

impl<'a> Command for Multi<'a> {
    fn execute (&mut self) -> Result<String, &'static str> {
        self.context.set_multi_mode_on();
        Ok(create_simple_string("OK"))
    }
}