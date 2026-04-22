mod ping_command;
mod echo_command;
mod set_command;
mod get_command;
mod lpush_command;
mod rpush_command;
mod lrange_command;
mod llen_command;
mod lpop_command;
mod blpop_command;
mod type_command;
mod xadd_command;
mod xrange;
mod xread;

use core::num;
use std::thread;
use std::time::Duration;
use std::{fmt::Display, str::FromStr, sync::Arc};
use chrono::Utc;

use crate::redis::command;
use crate::redis::command::xrange::Xrange;
use crate::redis::command::xread::Xread;
use crate::redis::resp::{self, create_array, create_bulk_string, create_empty_array, create_null_array, create_null_bulk_string, create_simple_integer};
use crate::redis::db::{self, DB, Value};

#[derive(Debug, PartialEq)]
pub enum RedisCommand {
    Ping, 
    Echo,
    Set,
    Get,
    Lpush,
    Rpush,
    Lrange,
    Llen,
    Lpop,
    Blpop,
    Type,
    Xadd,
    Xrange,
    Xread,
}

impl FromStr for RedisCommand {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ping" => Ok(RedisCommand::Ping),
            "echo" => Ok(RedisCommand::Echo),
            "set" => Ok(RedisCommand::Set),
            "get" => Ok(RedisCommand::Get),
            "lpush" => Ok(RedisCommand::Lpush),
            "rpush" => Ok(RedisCommand::Rpush),
            "lrange" => Ok(RedisCommand::Lrange),
            "llen" => Ok(RedisCommand::Llen),
            "lpop" => Ok(RedisCommand::Lpop),
            "blpop" => Ok(RedisCommand::Blpop),
            "type" => Ok(RedisCommand::Type),
            "xadd" => Ok(RedisCommand::Xadd),
            "xrange" => Ok(RedisCommand::Xrange),
            "xread" => Ok(RedisCommand::Xread),
            _ => Err(format!("unknown command: {}", s))
        }
    }
}

impl Display for RedisCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RedisCommand::Ping => write!(f, "ping"),
            RedisCommand::Echo => write!(f, "echo"),
            RedisCommand::Set => write!(f, "set"),
            RedisCommand::Get => write!(f, "get"),
            RedisCommand::Lpush => write!(f, "lpush"),
            RedisCommand::Rpush => write!(f, "rpush"),
            RedisCommand::Lrange => write!(f, "lrange"),
            RedisCommand::Llen => write!(f, "llen"),
            RedisCommand::Lpop => write!(f, "lpop"),
            RedisCommand::Blpop => write!(f, "blpop"),
            RedisCommand::Type => write!(f, "type"),
            RedisCommand::Xadd => write!(f, "xadd"),
            RedisCommand::Xrange => write!(f, "xrange"),
            RedisCommand::Xread => write!(f, "xread"),
        }
    }
}

impl RedisCommand {
    pub fn execute(command_array: &[&str]) -> Result<String, &'static str> {
        let command = command_array[0];
        let args = &command_array[1..];
        let command: Box<dyn Command> = match RedisCommand::from_str(command) {
            Ok(RedisCommand::Ping) => Box::new(ping_command::PingCommand{}),
            Ok(RedisCommand::Echo) => Box::new(echo_command::EchoCommand{args}),
            Ok(RedisCommand::Set) => Box::new(set_command::SetCommand{args}),
            Ok(RedisCommand::Get) => Box::new(get_command::GetCommand{args}),
            Ok(RedisCommand::Lpush) => Box::new(lpush_command::LpushCommand{args}),
            Ok(RedisCommand::Rpush) => Box::new(rpush_command::RpushCommand{args}),
            Ok(RedisCommand::Lrange) => Box::new(lrange_command::LrangeCommand{args}),
            Ok(RedisCommand::Llen) => Box::new(llen_command::LlenCommand{args}),
            Ok(RedisCommand::Lpop) => Box::new(lpop_command::LpopCommand{args}),
            Ok(RedisCommand::Blpop) => Box::new(blpop_command::BlPopCommand{args}),
            Ok(RedisCommand::Type) => Box::new(type_command::TypeCommand{args}),
            Ok(RedisCommand::Xadd) => Box::new(xadd_command::XaddCommand{args}),
            Ok(RedisCommand::Xrange) => Box::new(Xrange{args}),
            Ok(RedisCommand::Xread) => Box::new(Xread{args}),
            Err(_) => Box::new(InvalidCommand{}),
        };

        command.execute()     
    }
}

trait Command {
    fn execute (&self) -> Result<String, &'static str>;
}

pub struct InvalidCommand();

impl Command for InvalidCommand {
    fn execute (&self) -> Result<String, &'static str> {
        Err("INVALID COMMAND")
    }
}