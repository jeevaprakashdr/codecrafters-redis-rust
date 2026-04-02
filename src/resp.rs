use std::{fmt::format, io::Error, mem};

enum State {
    ArraySize,
    CRLF,
    BulkStringSize,
    BulkString,
}

pub fn process(cmd: &str) -> Result<Vec<String>, &'static str> {
    let parsed = parse(cmd);
    println!("{:?}", parsed);

    parsed
}

fn parse(input:&str) -> Result<Vec<String>, &'static str> {
    let mut tokens: Vec<String> = Vec::new();
    let mut token = String::new();
    let mut chars = input.chars();
    let mut state = State::ArraySize;

    let mut bulk_strings: Vec<String> = Vec::new();

    loop {
        let current = chars.next();
        state = match state {
            State::ArraySize => {                
                match current {
                    Some('*') => {
                        State::ArraySize
                    }
                    Some('1'..='9') => {
                        token.push(current.unwrap());
                        State::ArraySize
                    }
                    Some('\r') => {
                        tokens.push(mem::replace(&mut token, String::new()));
                        State::CRLF
                    }
                    _ => {
                        break
                    }
                }
            }
            State::CRLF => {
                match current {
                    Some('\n') => {
                        State::CRLF
                    }
                    Some('$') => {
                        State::BulkStringSize
                    }
                    _ => {
                        break
                    }
                }
            }
            State::BulkStringSize => {
                match current {
                    Some('0'..='9') => {
                        token.push(current.unwrap());
                        State::BulkStringSize
                    }
                    Some('\r') => { 
                        State::BulkStringSize
                    }
                    Some('\n') => { 
                        tokens.push(mem::replace(&mut token, String::new()));
                        State::BulkString
                    }
                    _ => {
                        break
                    }
                }
            }
            State::BulkString => {
                match current {
                    Some('a'..='z') | Some('A'..='Z') | Some('0'..='9') => {
                        token.push(current.unwrap());
                        State::BulkString
                    }
                    Some('\r') => {
                        State::BulkString
                    }
                    Some('\n') => {
                        if token.len() > 0 {
                            bulk_strings.push(token.clone());
                            tokens.push(mem::replace(&mut token, String::new()));
                        }
                     
                        State::BulkString
                    }
                    Some('$') => {
                        State::BulkStringSize
                    }
                    _ => {
                        break
                    }
                }
            }
        }
    }

    if token.len() > 0 {
        tokens.push(token);
    }
    
    Ok(bulk_strings)
}

pub fn create_simple_string(val: &str) -> String {
    format!("+{}\r\n", val)
}

pub fn create_bulk_string(val: &str) -> String {
    format!("${}\r\n{}\r\n", val.len(), val)
}

pub fn create_null_bulk_string() -> String {
    format!("$-1\r\n")
}

#[cfg(test)]
mod tests {
    use crate::resp::{
        create_simple_string, 
        create_bulk_string, 
        create_null_bulk_string,
        process};

    #[test]
    fn test_process_ping_command() {
        let ping_command = vec!["*1\r\n$4\r\nPING\r\n"];
        
        for cmd in ping_command {
            let result = process(cmd);
        
            assert!(result.is_ok(), "Cmd processing FAILED");
            let response = result.unwrap();
            assert_eq!(response, ["PING"]);
        }        
    }

    #[test]
    fn test_process_echo_command() {
        let ping_command = vec!["*2\r\n$4\r\nECHO\r\n$10\r\nstrawberry\r\n"];
        
        for cmd in ping_command {
            let result = process(cmd);
        
            assert!(result.is_ok(), "Cmd processing FAILED");
            let response = result.unwrap();
            assert_eq!(response, ["ECHO", "strawberry"]);
        }        
    }

    #[test]
    fn test_process_set_command() {
        let ping_command = vec!["*3\r\n$3\r\nSET\r\n$5\r\nmango\r\n$6\r\norange\r\n"];
        
        for cmd in ping_command {
            let result = process(cmd);
        
            assert!(result.is_ok(), "Cmd processing FAILED");
            let response = result.unwrap();
            assert_eq!(response, ["SET","mango", "orange"]);
        }        
    }

    #[test]
    fn test_process_set_command_with_expiry_setting() {
        let ping_command = vec!["*3\r\n$3\r\nSET\r\n$5\r\nmango\r\n$6\r\norange\r\n$2\r\nPX\r\n$3\r\n100\r\n"];
        
        for cmd in ping_command {
            let result = process(cmd);
        
            assert!(result.is_ok(), "Cmd processing FAILED");
            let response = result.unwrap();
            assert_eq!(response, ["SET","mango", "orange", "PX", "100"]);
        }        
    }

    #[test]
    fn test_process_get_command() {
        let ping_command = vec!["*3\r\n$3\r\nGET\r\n$5\r\nmango\r\n"];
        
        for cmd in ping_command {
            let result = process(cmd);
        
            assert!(result.is_ok(), "Cmd processing FAILED");
            let response = result.unwrap();
            assert_eq!(response, ["GET", "mango"]);
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
}
