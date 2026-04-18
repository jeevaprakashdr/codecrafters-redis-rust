use std::{str::FromStr, sync::Arc};

use crate::redis::{command::Command, db::{self, DB}, resp::create_empty_array, stream::{Stream, StreamEntryId}};

pub struct Xrange {
    pub args: Vec<String>
}

impl Command for Xrange {
    fn execute (&self) -> Result<String, &'static str> {
        let in_memory_db = Arc::clone(&DB);
        let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();

        let default  = StreamEntryId { ms: 0, seqno: 0 };
        let start = StreamEntryId::from_str(self.args[2].as_str()).unwrap_or(default);
        let end = StreamEntryId::from_str(self.args[3].as_str()).unwrap_or(default);
        match db.get_mut(self.args[1].to_string()) {
            Some(data) => {
                Ok(create_empty_array())
            },
            None => Ok(create_empty_array())
        }
    }
}