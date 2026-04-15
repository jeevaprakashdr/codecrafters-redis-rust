use std::{collections::HashMap, fmt::Display, str::FromStr, string};

use chrono::Utc;

#[derive(Debug, Clone)]
pub struct Stream {
    pub id : StreamEntryId,
    pub entries : HashMap<String, String>
}

impl Stream {
    pub fn new() -> Self {
        Self { 
            id: StreamEntryId::default(), 
            entries: HashMap::new() 
        }
    }
}

#[derive(Debug, PartialEq, Clone,)]
pub struct StreamEntry {
    pub key: String,
    pub value: String
}

impl Display for StreamEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.key, self.value)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct StreamEntryId {
    pub ms : i64,
    pub seqno : i64,
}
impl StreamEntryId {
    pub fn new(ms: i64, seqno: i64) -> Self {
        Self { ms, seqno }
    }
    
    pub fn auto_generate_seqno(&self) -> Self {
        Self {
            ms: self.ms,
            seqno: if self.ms == 0 { 1 } else { self.seqno },
        }
    }
    
    pub fn increment_seqno(&self) -> Self {
        Self {
            ms: self.ms,
            seqno: self.seqno + 1,
        }
    }
}

impl PartialOrd for StreamEntryId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.ms.partial_cmp(&other.ms) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.seqno.partial_cmp(&other.seqno)
    }

    fn gt(&self, other: &Self) -> bool {
        if self.ms != other.ms {
            self.ms > other.ms
        } else {
            self.seqno > other.seqno
        }
        
    }

    fn lt(&self, other: &Self) -> bool {
        if self.ms != other.ms  {
            self.ms < other.ms
        } else {
            self.seqno < other.seqno
        }
    }
}

impl Display for StreamEntryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.ms, self.seqno)
    }
}

impl Default for StreamEntryId {
    fn default() -> Self {
        let dt  = Utc::now();
        Self { ms: dt.timestamp_millis(), seqno: Default::default() }
    }
}

impl FromStr for StreamEntryId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "*" {
            return Ok(StreamEntryId::default())
        }

        let entry_id = s.split("-").collect::<Vec<_>>();
        if entry_id.len() != 2 {
            return Err(format!("failed"));
        }

        let ms = i64::from_str(entry_id[0]).unwrap_or(0);
        return Ok(StreamEntryId { 
            ms: ms, 
            seqno: if ms == 0 && entry_id[1] == "*" 
                { 1 } 
                else 
                { i64::from_str(entry_id[1]).unwrap_or(0) }
        });
    }
}

#[cfg(test)]
mod tests{
    use std::str::FromStr;

    use chrono::Utc;

    use crate::redis::stream::{StreamEntry, StreamEntryId};

    #[test]
    pub fn test_entry_id_create_from_autogenerate_string() {
        let stream_entry_id = StreamEntryId::from_str("*");

        assert!(stream_entry_id.is_ok());
        let entry_id = stream_entry_id.unwrap();
        
        assert!(entry_id.ms > 0);
        assert!(entry_id.seqno == 0);
    }

    #[test]
    pub fn test_entry_id_create_from_string() {
        let input = vec![
            ("0-0",StreamEntryId {ms:0,seqno:0}),
            ("0-1",StreamEntryId {ms:0,seqno:1}),
            ("1-0",StreamEntryId {ms:1,seqno:0}),
            ("1-1",StreamEntryId {ms:1,seqno:1}),
            ("0-*",StreamEntryId {ms:0,seqno:0}),
            ("1-*",StreamEntryId {ms:1,seqno:0}),
            ];

        for (ele, element_id) in input {
            let result = StreamEntryId::from_str(ele);
    
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), element_id);
        }
    }
    
    #[test]
    pub fn test_failed_entry_id_creating_from_string() {
        let input = vec!["", "00", "00-", "-00", "a-1", "1-a"];
        for ele in input {
            let result = StreamEntryId::from_str(ele);
            
            assert!(result.is_err())
        }
    }
    
    #[test]
    pub fn test_entry_id_convert_to_string() {
        let entra_id = StreamEntryId {
            ms: 0,
            seqno: 1
        };

        assert_eq!("0-1", entra_id.to_string())
    } 
    
    #[test]
    pub fn test_stream_convert_to_string() {
        let stream = StreamEntry {
            key: "key".to_string(),
            value: "value".to_string(),
        };

        assert_eq!("key:value", stream.to_string())
    }

    #[test]
    pub fn test_stream_entry_id_convert_to_string() {
        let stream_entry_id = StreamEntryId {
            ms: 0,
            seqno: 1
        };

        assert_eq!("0-1", stream_entry_id.to_string());
    }

    #[test]
    pub fn test_stream_entry_id_compare_two_stream_entry_id() {
        let input = vec![
            ((0,0),(1,0), false),
            ((0,0),(0,1), false),
            ((0,1),(0,1), false),
            ((1,0),(1,0), false),
            ((1,0),(0,0), true),
            ((0,1),(0,0), true),
            ];

        for ((ms1, seqno1), (ms2, seqno2), expected)  in input {
            let one = StreamEntryId { ms: ms1, seqno: seqno1};
            let two = StreamEntryId { ms: ms2, seqno: seqno2};
            
            assert_eq!(expected, one > two);    
        }
    }

    #[test]
    pub fn test_stream_entry_id_auto_generate_seqno() {
        let input = vec![
            ((0,0), (0,1)),
            ((1,0), (1,0))
            ];

        for ((ms, seqno), (expected_ms, expected_seqno)) in input {
            let stream_entry_id = StreamEntryId { ms, seqno};

            let actual = stream_entry_id.auto_generate_seqno();

            assert_eq!(actual, StreamEntryId {ms: expected_ms, seqno:expected_seqno});    
        }
    }
}