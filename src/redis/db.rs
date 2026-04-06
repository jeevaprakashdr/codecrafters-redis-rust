use std::{collections::HashMap, sync::{Arc, LazyLock, Mutex}};

use chrono::{DateTime, Utc};

pub struct Value {
    pub val : String,
    pub expire_at : Option<DateTime<Utc>>
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

    pub fn get(&self, key: String) -> Option<&Value> {
        self.data.get(key.as_str())
    }

     pub fn get_mut(&mut self, key: String) -> Option<&mut Value> {
        self.data.get_mut(key.as_str())
    }

    pub fn insert(&mut self, key: String, val: Value) -> Option<Value> {
        self.data.insert(key, val)
    }
}

pub static DB: LazyLock<Arc<Mutex<InMemoryDb>>> 
    = LazyLock::new(|| { Arc::new(Mutex::new(InMemoryDb::new()))});