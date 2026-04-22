use std::time::Duration;
use std::{str::FromStr, sync::Arc};
use std::{thread, time};

use crate::redis::command::Command;
use crate::redis::db::{self, DB};
use crate::redis::resp::{
    create_array, create_bulk_string, create_empty_array, create_null_array, create_resp_array,
};
use crate::redis::stream::{Stream, StreamEntryId};

pub struct Xread<'a> {
    pub args: &'a [&'a str]
}

impl<'a> Command for Xread<'a> {
    fn execute(&self) -> Result<String, &'static str> {
        let timeout = self.get_timeout();
        let mut timeout_expired = false;
        let mut previous = String::new();

        loop {
            if let Ok(data) = self.execute() {
                if previous.is_empty() && self.block_until_latest_record() {
                    previous = data;
                } else if !previous.is_empty() && previous != data {
                    return Ok(data);
                } else if !self.block_until_latest_record() {
                    return Ok(data);
                }
            }

            if timeout_expired {
                return Ok(create_null_array());
            }

            if timeout > 0 {
                thread::sleep(time::Duration::from_millis(timeout as u64));
                timeout_expired = true;
            }
        }
    }
}

impl<'a> Xread<'a> {
    fn execute(&self) -> Result<String, &'static str> {
        let stream_keys = self.get_arg_stream_keys();
        let start_entry_ids = self.get_arg_stream_entry_ids();

        let out: Vec<String> = stream_keys
            .iter()
            .enumerate()
            .map(|(index, key)| {
                if self.args.last() == Some(&"$") {
                    self.fetch_latest_data(key)
                } else {
                    self.fetch_data(
                        *start_entry_ids
                            .get(index)
                            .unwrap_or(&StreamEntryId { ms: 0, seqno: 0 }),
                        key,
                    )
                }
            })
            .filter(|vec| !vec.is_empty())
            .map(|stream_data| create_resp_array(&stream_data.iter().map(|f|f.as_str()).collect::<Vec<_>>()))
            .collect();

        if !out.is_empty() {
            Ok(create_resp_array(&out.iter().map(|f|f.as_str()).collect::<Vec<_>>()))
        } else {
            Err("No data found")
        }
    }

    fn fetch_latest_data(&self, stream_key: &String) -> Vec<String> {
        let in_memory_db = Arc::clone(&DB);
        let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();

        match db.get_mut(stream_key) {
            Some(data) => {
                if data.stream().is_empty() {
                    return vec![];
                }

                data
                    .stream()
                    .last()
                    .map(|stream| self.create_stream_entry_array(stream))
                    .map(|streams| create_resp_array(&streams.iter().map(|f|f.as_str()).collect::<Vec<_>>()))
                    .map(|stream_data| vec![stream_data])
                    .map(|stream_data| self.create_resp_stream_array(stream_key, &stream_data))
                    .unwrap_or_else(|| vec![])
            }
            None => vec![],
        }
    }

    fn fetch_data(&self, start: StreamEntryId, stream_key: &String) -> Vec<String> {
        let in_memory_db = Arc::clone(&DB);
        let mut db: std::sync::MutexGuard<'_, db::InMemoryDb> = in_memory_db.lock().unwrap();

        match db.get_mut(stream_key) {
            Some(data) => data
                .stream()
                .iter()
                .filter(|stream| stream.id > start)
                .collect::<Vec<_>>()
                .iter()
                .map(|stream| self.create_stream_entry_array(stream))
                .map(|streams| create_resp_array(&streams.iter().map(|f|f.as_str()).collect::<Vec<_>>()))
                .map(|stream_data| vec![stream_data])
                .flat_map(|stream_data| self.create_resp_stream_array(stream_key, &stream_data))
                .collect(),
            None => {
                vec![]
            }
        }
    }

    fn create_stream_entry_array(&self, stream: &Stream) -> Vec<String> {
        if stream.entries.is_empty() {
            return vec![];
        }

        let bs: String = create_bulk_string(stream.id.to_string().as_str());
        let entry_arr = create_array(
            &stream
                .entries
                .iter()
                .map(|val| val.as_str())
                .collect::<Vec<_>>(),
        );

        vec![bs, entry_arr]
    }

    fn create_resp_stream_array(&self, stream_key: &str, stream_data: &[String]) -> Vec<String> {
        vec![
            create_bulk_string(stream_key),
            create_resp_array(&stream_data.iter().map(|f| f.as_str()).collect::<Vec<_>>()),
        ]
    }

    fn has_blocking_request(&self) -> bool {
        self.args
            .iter()
            .any(|arg| arg.to_lowercase() == "block".to_string())
    }

    fn get_timeout(&self) -> usize {
        if self.has_blocking_request() {
            usize::from_str(self.args[1]).unwrap_or(0)
        } else {
            0
        }
    }

    fn block_until_latest_record(&self) -> bool {
        self.args.last() == Some(&"$")
    }

    fn get_arg_stream_keys(&self) -> Vec<String> {
        let start_index = if self.has_blocking_request() { 3 } else { 1 };
        self.args
            .iter()
            .skip(start_index)
            .filter(|arg| !arg.contains("-"))
            .map(|arg| arg.to_string())
            .collect::<Vec<_>>()
    }

    fn get_arg_stream_entry_ids(&self) -> Vec<StreamEntryId> {
        let start_index = if self.has_blocking_request() { 3 } else { 1 };
        self.args
            .iter()
            .skip(start_index)
            .filter(|arg| arg.contains("-"))
            .map(|arg| {
                StreamEntryId::from_str(arg).unwrap_or(StreamEntryId { ms: 0, seqno: 0 })
            })
            .collect::<Vec<_>>()
    }
}
