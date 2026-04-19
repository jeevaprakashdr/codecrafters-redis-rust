use std::{str::FromStr, sync::Arc};

use crate::redis::{command::Command, db::{self, DB}, resp::{create_array, create_bulk_string, create_empty_array}, stream::{Stream, StreamEntryId}};

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
                let filtered: Vec<&Stream> = data.stream
                    .iter()
                    .filter(|s| s.id >= start && s.id <= end)
                    .collect::<Vec<_>>();

                let out : Vec<String> = filtered
                    .iter()
                    .map(|f| {
                        let mut v = Vec::<String>::new();
                        v.push(create_bulk_string (f.id.to_string().as_str()));
                        v.push(create_array(&f.entries.iter().map(|x| x.as_str()).collect::<Vec<_>>()));
                        v
                    })
                    .map(|f|  format!("*{}\r\n{}", f.len(), f.join("")))
                    .collect();
                
                Ok(format!("*{}\r\n{}", out.len(), out.join("").to_string()))
            },
            None => Ok(create_empty_array())
        }
    }
}