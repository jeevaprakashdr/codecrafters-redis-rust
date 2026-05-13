use std::{mem, str::FromStr};

use crate::redis::commands::RedisCommand;

enum State {
    ArraySize,
    Crlf,
    BulkStringSize,
    BulkString,
}

pub(crate) fn parse(cmd_str: String) -> Result<RedisCommand, &'static str> {
    let parsed = parse_input(cmd_str);
    let args = parsed[1..].to_vec();
    match parsed.first().unwrap_or(&String::default()).to_lowercase().as_str() {
        "command" => Ok(RedisCommand::Command),
        "ping" => Ok(RedisCommand::Ping),
        "echo" => Ok(RedisCommand::Echo(args)),
        "set" => Ok(RedisCommand::Set(args)),
        "get" => Ok(RedisCommand::Get(args)),
        "lpush" => Ok(RedisCommand::Lpush(args)),
        "rpush" => Ok(RedisCommand::Rpush(args)),
        "lrange" => Ok(RedisCommand::Lrange(args)),
        "llen" => Ok(RedisCommand::Llen(args)),
        "lpop" => Ok(RedisCommand::Lpop(args)),
        "blpop" => Ok(RedisCommand::Blpop(args)),
        "type" => Ok(RedisCommand::Type(args)),
        "xadd" => Ok(RedisCommand::Xadd(args)),
        "xrange" => Ok(RedisCommand::Xrange(args)),
        "xread" => Ok(RedisCommand::Xread(args)),
        "incr" => Ok(RedisCommand::Incr(args)),
        "multi" => Ok(RedisCommand::Multi),
        "exec" => Ok(RedisCommand::Exec),
        "discard" => Ok(RedisCommand::Discard),
        "info" => Ok(RedisCommand::Info),
        "replconf" => Ok(RedisCommand::Replconf(args)),
        "psync" => Ok(RedisCommand::Psync),
        _ => Ok(RedisCommand::InvalidCommand),
    }
}

fn parse_input(input: String) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    let mut token = String::new();
    let mut chars = input.chars();
    let mut state = State::ArraySize;

    let mut parsed_val: Vec<String> = Vec::new();

    loop {
        let current = chars.next();
        state = match state {
            State::ArraySize => match current {
                Some('*') => State::ArraySize,
                Some('0'..='9') => {
                    token.push(current.unwrap());
                    State::ArraySize
                }
                Some('\r') => {
                    tokens.push(mem::take(&mut token));
                    State::Crlf
                }
                _ => break,
            },
            State::Crlf => match current {
                Some('\n') => State::Crlf,
                Some('$') => State::BulkStringSize,
                _ => break,
            },
            State::BulkStringSize => match current {
                Some('0'..='9') => {
                    token.push(current.unwrap());
                    State::BulkStringSize
                }
                Some('$') => State::BulkStringSize,
                Some('\r') => State::BulkStringSize,
                Some('\n') => {
                    tokens.push(mem::take(&mut token));
                    State::BulkString
                }
                _ => break,
            },
            State::BulkString => match current {
                Some('a'..='z') | Some('A'..='Z') | Some('0'..='9') | Some('_') | Some('-')
                | Some('.') | Some('*') | Some('+') | Some('$') => {
                    token.push(current.unwrap());
                    State::BulkString
                }
                Some('\r') => State::BulkString,
                Some('\n') => {
                    if !token.is_empty() {
                        parsed_val.push(token.clone());
                        tokens.push(mem::take(&mut token));
                    }

                    State::BulkStringSize
                }
                _ => break,
            },
        }
    }

    if !token.is_empty() {
        tokens.push(token);
    }

    println!("{:?}", parsed_val);
    parsed_val
}

pub fn create_array(collection: &[&str]) -> String {
    let collection_string = collection
        .iter()
        .map(|r| r.to_string())
        .collect::<Vec<_>>()
        .join("");
    format!("*{}\r\n{}", collection.iter().len(), collection_string)
}

pub fn create_array_bulk_string(collection: &[&str]) -> String {
    let collection_string = collection
        .iter()
        .map(|r| create_bulk_string(r))
        .collect::<Vec<_>>()
        .join("");
    format!("*{}\r\n{}", collection.iter().len(), collection_string)
}

pub fn create_null_array() -> String {
    "*-1\r\n".to_string()
}

pub fn create_empty_array() -> String {
    "*0\r\n".to_string()
}

pub fn create_simple_integer(val: usize) -> String {
    format!(":{}\r\n", val)
}

pub fn create_simple_string(val: &str) -> String {
    format!("+{}\r\n", val)
}

pub fn create_bulk_string(val: &str) -> String {
    format!("${}\r\n{}\r\n", val.len(), val)
}

pub fn create_null_bulk_string() -> String {
    "$-1\r\n".to_string()
}

pub fn create_resp_array(val: &[&str]) -> String {
    format!("*{}\r\n{}", val.len(), val.join(""))
}

#[cfg(test)]
mod tests {
    use crate::redis::resp::{
        create_array_bulk_string, create_bulk_string, create_empty_array, create_null_array,
        create_null_bulk_string, create_simple_integer, create_simple_string, parse_input,
    };

    #[test]
    fn test_create_simple_integer() {
        let val = 123;

        let simple_integer = create_simple_integer(val);

        assert_eq!(format!(":{}\r\n", val), simple_integer)
    }

    #[test]
    fn test_parse_get_command() {
        let inputs: Vec<(&str, &[&str])> = vec![
            ("*3\r\n$3\r\nGET\r\n$5\r\nmango\r\n", &["GET", "mango"]),
            (
                "*3\r\n$3\r\nGET\r\n$5\r\nmango_1_2\r\n",
                &["GET", "mango_1_2"],
            ),
            ("*3\r\n$3\r\nGET\r\n$2\r\n-1\r\n", &["GET", "-1"]),
            (
                "*3\r\n$3\r\nSET\r\n$5\r\nmango\r\n$6\r\norange\r\n$2\r\nPX\r\n$3\r\n100\r\n",
                &["SET", "mango", "orange", "PX", "100"],
            ),
            (
                "*10\r\n$5\r\nRPUSH\r\n$4\r\npear\r\n$9\r\nraspberry\r\n$9\r\npineapple\r\n$5\r\ngrape\r\n$9\r\nblueberry\r\n$5\r\nmango\r\n$6\r\norange\r\n$10\r\nstrawberry\r\n$6\r\nbanana\r\n",
                &[
                    "RPUSH",
                    "pear",
                    "raspberry",
                    "pineapple",
                    "grape",
                    "blueberry",
                    "mango",
                    "orange",
                    "strawberry",
                    "banana",
                ],
            ),
            (
                "*5\r\n$4\r\nXADD\r\n$9\r\nblueberry\r\n$3\r\n0-*\r\n$5\r\ngrape\r\n$5\r\napple\r\n",
                &["XADD", "blueberry", "0-*", "grape", "apple"],
            ),
            (
                "*4\r\n$6\r\nXRANGE\r\n$5\r\napple\r\n$3\r\n0-2\r\n$1\r\n+\r\n",
                &["XRANGE", "apple", "0-2", "+"],
            ),
        ];

        for (cmd, expected) in inputs {
            let result: Vec<String> = parse_input(cmd.to_string());

            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_create_simple_string() {
        let val = "text";

        let simple_string = create_simple_string(val);

        assert_eq!(format!("+{}\r\n", val), simple_string)
    }

    #[test]
    fn test_create_bulk_string() {
        let val = "text";

        let bulk_string = create_bulk_string(val);

        assert_eq!(format!("${}\r\n{}\r\n", val.len(), val), bulk_string)
    }

    #[test]
    fn test_create_null_bulk_string() {
        let null_bulk_string = create_null_bulk_string();
        assert_eq!("$-1\r\n", null_bulk_string)
    }

    #[test]
    fn test_create_empty_array() {
        assert_eq!("*0\r\n".to_string(), create_empty_array())
    }

    #[test]
    fn test_create_null_array() {
        assert_eq!("*-1\r\n".to_string(), create_null_array())
    }

    #[test]
    fn test_create_array() {
        let input: Vec<(&[&str], &str)> = vec![
            (&["a"], "*1\r\n$1\r\na\r\n"),
            (&["a", "b"], "*2\r\n$1\r\na\r\n$1\r\nb\r\n"),
        ];

        for (arr, expected) in input {
            let actual = create_array_bulk_string(arr);

            assert_eq!(expected.to_string(), actual);
        }
    }
}
