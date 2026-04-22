use crate::redis::{command::Command, db::{self, DB}, resp::{create_array, create_null_array}};
use std::{fmt::Display, str::FromStr, sync::Arc, thread, time::Duration};

pub struct BlPopCommand<'a> {
    pub args: &'a [&'a str]
}

impl<'a> Command for BlPopCommand<'a> {
    fn execute (&self) -> Result<String, &'static str> {
       execute_blpop(self.args)
    }
}

fn execute_blpop(args: &[&str]) -> Result<String, &'static str> { 
    let timeout = args
                .get(1)
                .map(|f |f64::from_str(f).unwrap_or(0.0))
                .unwrap_or(0.0);
    
    let mut timeout_expired = false;

    loop {
        if let Some(popped) = blpop(args) {
            return Ok(create_array(&[args[0], popped.as_str()]))
        }

        if timeout_expired {
            return Ok(create_null_array())
        }

        if timeout > 0.0 {
            thread::sleep(Duration::from_secs_f64(timeout));
            timeout_expired = true;
        }
    }    
}

fn blpop(args: &[&str]) -> Option<String> {
    let in_memory_db = Arc::clone(&DB);
    let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();
    match db.get_mut(args[0]) {
        Some(data) => {
            match data.list().to_vec().split_first() {
                Some((first, rest)) => {
                    data.set_list(rest);
                    Some(first.to_string())
                }
                None => None,
            }
        }
        None => None
    }
}