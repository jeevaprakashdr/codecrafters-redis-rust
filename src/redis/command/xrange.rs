use std::{str::FromStr, sync::Arc};

use crate::redis::{command::Command, db::{self, DB}, resp::{create_array, create_bulk_string, create_empty_array}, stream::{Stream, StreamEntryId}};

pub struct Xrange<'a> {
    pub args: &'a [&'a str]
}

impl<'a> Command for Xrange<'a> {
    fn execute (&self) -> Result<String, &'static str> {
        let in_memory_db = Arc::clone(&DB);
        let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();

        let default  = StreamEntryId { ms: 0, seqno: 0 };
        let start = StreamEntryId::from_str(self.args[1]).unwrap_or(default);
        let end = StreamEntryId::from_str(self.args[2]).unwrap_or(default);
        let filter_till_end_of_stream = self.args[2] == "+";
        match db.get_mut(self.args[0].to_string()) {
            Some(data) => {
                let filtered: Vec<&Stream> = data.stream()
                    .iter()
                    .filter(|stream| 
                        stream.id >= start && (filter_till_end_of_stream || stream.id <= end))
                    .collect::<Vec<_>>();

                let out : Vec<String> = filtered
                    .iter()
                    .map(|stream| {
                        let bs = create_bulk_string (stream.id.to_string().as_str());
                        let arr = create_array(&stream.entries.iter().map(|val| val.as_str()).collect::<Vec<_>>());
                        vec![bs, arr]
                    })
                    .map(|streams|  format!("*{}\r\n{}", streams.len(), streams.join("")))
                    .collect();
                
                Ok(format!("*{}\r\n{}", out.len(), out.join("")))
            },
            None => Ok(create_empty_array())
        }
    }
}