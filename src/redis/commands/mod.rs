mod ping;
mod echo;
mod set;
mod get;
mod lpush;
mod rpush;
mod lrange;
mod llen;
mod lpop;
mod blpop_command;
mod r#type;
mod xadd;
mod xrange;
mod xread;
mod incr;
mod multi;
mod exec;

use core::num;
use std::thread;
use std::time::Duration;
use std::{fmt::Display, str::FromStr, sync::Arc};
use chrono::Utc;

use crate::redis::commands;
use crate::redis::commands::blpop_command::BlPopCommand;
use crate::redis::commands::echo::Echo;
use crate::redis::commands::exec::Exec;
use crate::redis::commands::get::Get;
use crate::redis::commands::incr::Incr;
use crate::redis::commands::llen::Llen;
use crate::redis::commands::lpop::Lpop;
use crate::redis::commands::lpush::Lpush;
use crate::redis::commands::lrange::Lrange;
use crate::redis::commands::multi::Multi;
use crate::redis::commands::ping::Ping;
use crate::redis::commands::rpush::Rpush;
use crate::redis::commands::set::Set;
use crate::redis::commands::r#type::Type;
use crate::redis::commands::xadd::Xadd;
use crate::redis::commands::xrange::Xrange;
use crate::redis::commands::xread::Xread;
use crate::redis::resp::{self, create_array_bulk_string, create_bulk_string, create_empty_array, create_null_array, create_null_bulk_string, create_simple_integer};
use crate::redis::db::{self, DB, Value};
use crate::redis::settings::RedisSetting;

#[derive(Debug, PartialEq)]
pub enum RedisCommand {
    Command,
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
    Incr,
    Multi,
    Exec,
}

impl FromStr for RedisCommand {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "command" => Ok(RedisCommand::Command),
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
            "incr" => Ok(RedisCommand::Incr),
            "multi" => Ok(RedisCommand::Multi),
            "exec" => Ok(RedisCommand::Exec),
            _ => Err(format!("unknown command: {}", s))
        }
    }
}

impl RedisCommand {
    pub fn execute(redis_setting: Arc<std::sync::Mutex<RedisSetting>>, command_array: &[&str])
        -> Result<String, &'static str> {
        let command = command_array[0];
        let args = &command_array[1..];
        let command = Self::create_command(redis_setting, command, args);
        command.execute()     
    }

    fn create_command<'a>(
        redis_setting: Arc<std::sync::Mutex<RedisSetting>>,
        command: &str,
        args: &'a [&'a str]) -> Box<dyn Command + 'a> {
        match RedisCommand::from_str(command) {
            Ok(RedisCommand::Command) => Box::new(Ping{}),
            Ok(RedisCommand::Ping) => Box::new(Ping{}),
            Ok(RedisCommand::Echo) => Box::new(Echo{args}),
            Ok(RedisCommand::Set) => Box::new(Set{args, redis_setting}),
            Ok(RedisCommand::Get) => Box::new(Get{args, redis_setting}),
            Ok(RedisCommand::Lpush) => Box::new(Lpush{args}),
            Ok(RedisCommand::Rpush) => Box::new(Rpush{args}),
            Ok(RedisCommand::Lrange) => Box::new(Lrange{args}),
            Ok(RedisCommand::Llen) => Box::new(Llen{args}),
            Ok(RedisCommand::Lpop) => Box::new(Lpop{args}),
            Ok(RedisCommand::Blpop) => Box::new(BlPopCommand{args}),
            Ok(RedisCommand::Type) => Box::new(Type{args}),
            Ok(RedisCommand::Xadd) => Box::new(Xadd{args}),
            Ok(RedisCommand::Xrange) => Box::new(Xrange{args}),
            Ok(RedisCommand::Xread) => Box::new(Xread{args}),
            Ok(RedisCommand::Incr) => Box::new(Incr{args, redis_setting}),
            Ok(RedisCommand::Multi) => Box::new(Multi{redis_setting}),
            Ok(RedisCommand::Exec) => Box::new(Exec{redis_setting}),
            Err(_) => Box::new(InvalidCommand{}),
        }
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