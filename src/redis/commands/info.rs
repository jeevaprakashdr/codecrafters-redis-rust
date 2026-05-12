use std::sync::Arc;

use crate::redis::resp::create_bulk_string;
use crate::redis::commands::Command;
use crate::redis::server::ServerContext;

pub struct Info {
    pub server_context: Arc<std::sync::Mutex<ServerContext>>,
}

impl Command for Info {
    fn execute (&mut self) -> Result<String, &'static str> {
        let server_context = self.server_context.lock().unwrap();
        
        Ok(create_bulk_string(server_context.get_role_info().join("\n").as_str()))
    }
}
