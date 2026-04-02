use chrono::{DateTime, Utc};

pub struct Value {
    pub val : String,
    pub expire_at : Option<DateTime<Utc>>
}