mod blpop_command;
mod discard;
mod echo;
mod exec;
mod get;
mod incr;
mod llen;
mod lpop;
mod lpush;
mod lrange;
mod multi;
mod ping;
mod rpush;
mod set;
mod r#type;
mod xadd;
mod xrange;
mod xread;
mod info;

use chrono::Utc;
use std::collections::VecDeque;
use std::io::Write;
use std::net::TcpStream;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use crate::redis::commands;
use crate::redis::commands::blpop_command::BlPopCommand;
use crate::redis::commands::discard::Discard;
use crate::redis::commands::echo::Echo;
use crate::redis::commands::exec::Exec;
use crate::redis::commands::get::Get;
use crate::redis::commands::incr::Incr;
use crate::redis::commands::info::Info;
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
use crate::redis::db::{self, DB, Value};
use crate::redis::resp::{
    self, create_array_bulk_string, create_bulk_string, create_empty_array, create_null_array,
    create_null_bulk_string, create_simple_integer,
};
use crate::redis::server::ServerContext;

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
    Discard,
    Info,
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
            "discard" => Ok(RedisCommand::Discard),
            "info" => Ok(RedisCommand::Info),
            _ => Err(format!("unknown command: {}", s)),
        }
    }
}

impl RedisCommand {
    fn create<'a>(
        cmd_str: String, 
        args: Vec<String>, 
        context: &'a mut CommandHandlerContext,
        server_context: Arc<std::sync::Mutex<ServerContext>>
    ) 
    -> Box<dyn Command + 'a> {
        match RedisCommand::from_str(cmd_str.as_str()) {
            Ok(RedisCommand::Command) => Box::new(Ping {}),
            Ok(RedisCommand::Ping) => Box::new(Ping {}),
            Ok(RedisCommand::Echo) => Box::new(Echo { args }),
            Ok(RedisCommand::Set) => Box::new(Set {
                context,
                args,
            }),
            Ok(RedisCommand::Get) => Box::new(Get {
                context,
                args,
            }),
            Ok(RedisCommand::Lpush) => Box::new(Lpush { args }),
            Ok(RedisCommand::Rpush) => Box::new(Rpush { args }),
            Ok(RedisCommand::Lrange) => Box::new(Lrange { args }),
            Ok(RedisCommand::Llen) => Box::new(Llen { args }),
            Ok(RedisCommand::Lpop) => Box::new(Lpop { args }),
            Ok(RedisCommand::Blpop) => Box::new(BlPopCommand { args }),
            Ok(RedisCommand::Type) => Box::new(Type { args }),
            Ok(RedisCommand::Xadd) => Box::new(Xadd { args }),
            Ok(RedisCommand::Xrange) => Box::new(Xrange { args }),
            Ok(RedisCommand::Xread) => Box::new(Xread { args }),
            Ok(RedisCommand::Incr) => Box::new(Incr {
                context,
                args,
            }),
            Ok(RedisCommand::Multi) => Box::new(Multi { context }),
            Ok(RedisCommand::Exec) => Box::new(Exec { server_context, context }),
            Ok(RedisCommand::Discard) => Box::new(Discard { context, }),
            Ok(RedisCommand::Info) => Box::new(Info { server_context }),
            Err(_) => Box::new(InvalidCommand {}),
        }
    }
}

trait Command {
    fn execute(&mut self) -> Result<String, &'static str>;
}

pub struct InvalidCommand();

impl Command for InvalidCommand {
    fn execute(&mut self) -> Result<String, &'static str> {
        Err("INVALID COMMAND")
    }
}

pub(crate) struct QueueContent {
    pub command_str: String,
    pub args: Vec<String>,
}

pub(crate) struct CommandHandlerContext {
    multi_mode: bool,
    queue: VecDeque<QueueContent>,
}

impl CommandHandlerContext {
    pub(crate) fn new(multi_mode: bool, queue: VecDeque<QueueContent>) -> Self {
        Self { multi_mode, queue }
    }
    
    fn is_multi_mode_on(&self) -> bool {
        self.multi_mode
    }
    
    fn set_multi_mode_off(&mut self) {
        self.multi_mode = false
    }

    fn set_multi_mode_on(&mut self) {
        self.multi_mode = true
    }

    fn flush_queue(&mut self) {
        self.queue.clear()
    }
    
    fn push(&mut self, command: QueueContent) {
        self.queue.push_back(command)
    }
}

pub(crate) struct CommandHandler<'a> {
    cmd_str: String,
    args: Vec<String>,
    context: &'a mut CommandHandlerContext,
}

impl<'a> CommandHandler<'a> {
    pub(crate) fn new(cmd_str: String, args: Vec<String>, context: &'a mut CommandHandlerContext) -> Self {
        Self {
            cmd_str,
            args,
            context
        }
    }

    pub(crate) fn handle(&mut self, mut stream: &TcpStream, server_context: Arc<std::sync::Mutex<ServerContext>>) {
        let mut command = RedisCommand::create(self.cmd_str.clone(), self.args.clone(), self.context, server_context);
        let result = command.execute()
            .inspect_err(|err| eprintln!("{}", err))
            .unwrap_or_else(|e| e.to_string());
        stream.write_all(result.as_bytes()).unwrap();
    }
}
