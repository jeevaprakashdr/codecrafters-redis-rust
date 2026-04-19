use std::{str::FromStr, sync::Arc};

use crate::redis::{command::Command, db::{self, DB}, resp::{create_array, create_bulk_string, create_empty_array, create_simple_string}, stream::{Stream, StreamEntryId}};

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
                let out: Vec<&Stream> = data.stream
                    .iter()
                    .filter(|s| s.id >= start && s.id <= end)
                    .collect::<Vec<_>>();

                let mut actualout = Vec::<Vec<String>>::new();
                for ele in out {
                    let mut v = Vec::<String>::new();
                    v.push(create_bulk_string (ele.id.to_string().as_str()));
                    v.push(create_array(&ele.entries.iter().map(|f| f.as_str()).collect::<Vec<_>>()));
                    actualout.push(v);
                }
                
                
                let k: Vec<String> = actualout.iter().map(|f| {
                    f.join("")
                }).collect();
                
                //println!("{}", format!("*{}\r\n{:?}", k.len(), k.join("")));
                Ok(format!("*{}\r\n{:?}", k.len(), k.join("")))
            },
            None => Ok(create_empty_array())
        }
    }
}