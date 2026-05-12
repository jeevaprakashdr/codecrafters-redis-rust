use std::sync::Arc;

use crate::redis::server::ServerContext;
use crate::redis::resp::create_simple_string;
use crate::redis::commands::{Command, CommandHandlerContext};

pub struct Discard<'a> {
    pub context: &'a mut CommandHandlerContext,
}

impl<'a> Command for Discard<'a> {
    fn execute (&mut self) -> Result<String, &'static str> {
        if !self.context.is_multi_mode_on(){
            return Ok("-ERR DISCARD without MULTI\r\n".to_string())
        }

        self.context.flush_queue();
        self.context.set_multi_mode_off();

        Ok(create_simple_string("OK"))
    }
}

