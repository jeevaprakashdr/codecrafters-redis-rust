use crate::redis::resp::{create_bulk_string};
use crate::redis::commands::Command;

pub struct Info { }


impl Command for Info {
    fn execute (&self) -> Result<String, &'static str> {
        Ok(create_bulk_string("role:master"))
    }
}
