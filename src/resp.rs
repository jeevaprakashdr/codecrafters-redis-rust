use std::{fmt::format, io::Error, mem};

enum State {
    ArraySize,
    CRLF,
    BulkStringSize,
    BulkString,
}

pub fn process(cmd: &str) -> Result<String, &'static str> {
    let parsed = parse(cmd);
    println!("{:?}", parsed);

    match parsed {
        Ok((_, bulk_strings)) => {
            match bulk_strings[0].to_lowercase().as_str() {
                "ping" => Result::Ok(create_simple_string("PONG")),
                "echo" => Result::Ok(create_bulk_string( bulk_strings[1..].join(" ").as_str())),
                _ => Err("Invalid command")
            }
        }
        Err(_) => {
            Err("Invalid command")
        }
    }    
}

fn parse(input:&str) -> Result<(Vec<String>, Vec<String>), &'static str> {
    let mut tokens: Vec<String> = Vec::new();
    let mut token = String::new();
    let mut chars = input.chars();
    let mut state = State::ArraySize;

    let commands: Vec<String> = Vec::new();
    let mut bulk_strings: Vec<String> = Vec::new();

    loop {
        let current = chars.next();
        state = match state {
            State::ArraySize => {                
                match current {
                    Some('*') => {
                        State::ArraySize
                    }
                    // todo unsure if we need this for now 
                    // Some('0') => {
                    //     State::ArraySize
                    // }
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
                    Some('1'..='9') => {
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
                    Some('a'..='z') | Some('A'..='Z') => {
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
    
    Ok((commands, bulk_strings))
}

fn create_simple_string(val: &str) -> String {
    format!("+{}\r\n", val)
}

fn create_bulk_string(val: &str) -> String {
    format!("${}\r\n{}\r\n", val.len(), val)
}

#[cfg(test)]
mod tests {
    use crate::resp::{create_simple_string, create_bulk_string, process};

    #[test]
    fn test_process_ping_command() {
        let ping_command = vec!["*1\r\n$4\r\nPING\r\n"];
        
        for cmd in ping_command {
            let result = process(cmd);
        
            assert!(result.is_ok(), "Cmd processing FAILED");
            let response = result.unwrap();
            assert_eq!(response.as_str(), "+PONG\r\n")    
        }        
    }

    #[test]
    fn test_process_echo_command() {
        let ping_command = vec!["*2\r\n$4\r\nECHO\r\n$5\r\nmango\r\n"];
        
        for cmd in ping_command {
            let result = process(cmd);
        
            assert!(result.is_ok(), "Cmd processing FAILED");
            let response = result.unwrap();
            assert_eq!(response.as_str(), "$5\r\nmango\r\n")    
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
}
