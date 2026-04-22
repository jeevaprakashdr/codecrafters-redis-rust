use std::{collections::HashMap, sync::{Arc, LazyLock, Mutex}};

use chrono::{DateTime, Utc};

use crate::redis::stream::{Stream, StreamEntryId};

#[derive(Debug)]
pub struct Value {
    str : String,
    list : Vec<String>,
    stream : Vec<Stream>,
    expire_at : Option<DateTime<Utc>>,
}

impl Value {
    pub fn new(str: String, list : Vec<String>, stream : Vec<Stream>, expire_at : Option<DateTime<Utc>>) -> Self {
        Self {
                str,
                list,
                stream,
                expire_at, 
        }
    }

    pub fn with_str(str: String) -> Self {
        Value::new(str, vec![], vec![], None)
    }

    pub fn with_list(list : Vec<String>) -> Self {
        Value::new(String::new(), list, vec![], None)
    }

    pub fn with_stream(stream : Vec<Stream>) -> Self {
        Value::new(String::new(), vec![], stream, None)
    }

    pub fn str_val(&self) -> &str { &self.str }
    pub fn list(&self) -> &[String] { &self.list }
    pub fn set_list(&mut self, list: &[String]) { 
        self.list = list.to_vec()
    }
    
    pub fn stream(&self) -> &[Stream] { &self.stream }
    pub fn push_stream(&mut self, stream: Stream) { 
        self.stream.push(stream)
    }

    pub fn expire_at(&self) -> Option<DateTime<Utc>> { self.expire_at }
    pub fn set_expire_at(&mut self, expire_at : DateTime<Utc>) { 
        self.expire_at = Some(expire_at)
    }
}

pub struct InMemoryDb {
    data:  HashMap<String, Value>
}

impl InMemoryDb {
    pub fn new() -> Self {
        Self {
            data: HashMap::new()
        }
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

     pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        self.data.get_mut(key)
    }

    pub fn insert(&mut self, key: &str, val: Value) -> Option<Value> {
        self.data.insert(key.to_string(), val)
    }
}

pub static DB: LazyLock<Arc<Mutex<InMemoryDb>>> 
    = LazyLock::new(|| { Arc::new(Mutex::new(InMemoryDb::new()))});