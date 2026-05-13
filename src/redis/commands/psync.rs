use std::sync::Arc;

use crate::redis::{commands::Command, resp::create_simple_string, server::ServerContext};

pub struct Psync {
    pub server_context: Arc<std::sync::Mutex<ServerContext>>,
}

impl Command for Psync {
    fn execute(&mut self) -> Result<String, &'static str> {
        let server_context = self.server_context.lock().unwrap();
        let id= server_context.get_replication_id();

        Ok(create_simple_string(format!("FULLRESYNC {} 0", id).as_str()))
    }
}