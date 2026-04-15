use std::{collections::HashMap, str::FromStr, sync::Arc};

use crate::redis::{command::Command, db::{self, DB, Value}, resp::create_bulk_string, stream::{Stream, StreamEntryId}};


pub struct XaddCommandNew {
    pub args: Vec<String>
}

impl Command for XaddCommandNew {
    fn execute (&self) -> Result<String, &'static str> {
        let in_memory_db = Arc::clone(&DB);
        let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
        
        match db.get_mut(self.args[1].to_string()) {
            Some(_) => todo!(),
            None => {
                let (arg_key_val, _r) = self.args.as_chunks::<2>();
                let mut entries : HashMap<String, String> = HashMap::new();
                for ele in arg_key_val.iter().skip(1) {
                    entries.insert(ele[0].to_owned(), ele[1].to_owned());
                }

                let key = self.args[1].to_string();
                let stream_entry_id = StreamEntryId::from_str(self.args[2].as_str()).unwrap();
                let val  = Value {
                    val: "".to_string(),
                    data_type: None,
                    expire_at: None, 
                    stream: Some(Stream {
                        id: stream_entry_id,
                        entries,
                    }),
                };

                db.insert(key, val);
                Ok(create_bulk_string(stream_entry_id.to_string().as_str()))
            },
        }
        
        
    }
}