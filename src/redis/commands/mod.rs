mod invalid;
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
mod replconf;
mod psync;

use chrono::Utc;
use std::collections::VecDeque;
use std::fmt::Display;
use std::io::Write;
use std::net::TcpStream;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use crate::redis::commands;
use crate::redis::commands::invalid::InvalidCommand;
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
use crate::redis::commands::psync::Psync;
use crate::redis::commands::replconf::Replconf;
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

#[derive(Clone, Debug, PartialEq)]
pub enum RedisCommand {
    InvalidCommand,
    Command,
    Ping,
    Echo(Vec<String>),
    Set(Vec<String>),
    Get(Vec<String>),
    Lpush(Vec<String>),
    Rpush(Vec<String>),
    Lrange(Vec<String>),
    Llen(Vec<String>),
    Lpop(Vec<String>),
    Blpop(Vec<String>),
    Type(Vec<String>),
    Xadd(Vec<String>),
    Xrange(Vec<String>),
    Xread(Vec<String>),
    Incr(Vec<String>),
    Multi,
    Exec,
    Discard,
    Info,
    Replconf(Vec<String>),
    Psync
}

impl Display for RedisCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cmd = match self {
            RedisCommand::InvalidCommand => "invalid",
            RedisCommand::Ping => "ping",
            RedisCommand::Command => "command",
            RedisCommand::Echo(_) => "echo",
            RedisCommand::Set(_) => "set",
            RedisCommand::Get(_) => "get",
            RedisCommand::Lpush(_) => "lpush",
            RedisCommand::Rpush(_) => "rpush",
            RedisCommand::Lrange(_) => "lrange",
            RedisCommand::Llen(_) => "llen",
            RedisCommand::Lpop(_) => "lpop",
            RedisCommand::Blpop(_) => "blpop",
            RedisCommand::Type(_) => "type",
            RedisCommand::Xadd(_) => "xadd",
            RedisCommand::Xrange(_) => "xrange",
            RedisCommand::Xread(_) => "xread",
            RedisCommand::Incr(_) => "incr",
            RedisCommand::Multi => "multi",
            RedisCommand::Exec => "exec",
            RedisCommand::Discard => "discard",
            RedisCommand::Info => "info",
            RedisCommand::Replconf(_) => "replconf",
            RedisCommand::Psync => "psync",
        };

        write!(f, "{}", cmd.to_uppercase())
    }
}

impl RedisCommand {
    fn create<'a>(
        command: RedisCommand, 
        context: &'a mut CommandHandlerContext,
        server_context: Arc<std::sync::Mutex<ServerContext>>
    ) 
    -> Box<dyn Command + 'a> {
        match command {
            RedisCommand::InvalidCommand => Box::new(InvalidCommand{}),
            RedisCommand::Command => Box::new(Ping {}),
            RedisCommand::Ping => Box::new(Ping {}),
            RedisCommand::Echo(args) => Box::new(Echo { args }),
            RedisCommand::Set(args) => Box::new(Set {
                context,
                args,
            }),
            RedisCommand::Get(args) => Box::new(Get {
                context,
                args,
            }),
            RedisCommand::Lpush(args) => Box::new(Lpush { args }),
            RedisCommand::Rpush(args) => Box::new(Rpush { args }),
            RedisCommand::Lrange(args) => Box::new(Lrange { args }),
            RedisCommand::Llen(args) => Box::new(Llen { args }),
            RedisCommand::Lpop(args) => Box::new(Lpop { args }),
            RedisCommand::Blpop(args) => Box::new(BlPopCommand { args }),
            RedisCommand::Type(args) => Box::new(Type { args }),
            RedisCommand::Xadd(args) => Box::new(Xadd { args }),
            RedisCommand::Xrange(args) => Box::new(Xrange { args }),
            RedisCommand::Xread(args) => Box::new(Xread { args }),
            RedisCommand::Incr(args) => Box::new(Incr {
                context,
                args,
            }),
            RedisCommand::Multi => Box::new(Multi { context }),
            RedisCommand::Exec => Box::new(Exec { server_context, context }),
            RedisCommand::Discard => Box::new(Discard { context, }),
            RedisCommand::Info => Box::new(Info { server_context }),
            RedisCommand::Replconf(args) => Box::new(Replconf{ args }),
            RedisCommand::Psync => Box::new(Psync{ server_context }),
        }
    }
}

trait Command {
    fn execute(&mut self) -> Result<String, &'static str>;
}

pub(crate) struct QueueContent {
    pub command: RedisCommand,
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
    command: RedisCommand,
    context: &'a mut CommandHandlerContext,
}

impl<'a> CommandHandler<'a> {
    pub(crate) fn new(command: RedisCommand, context: &'a mut CommandHandlerContext) -> Self {
        Self {
            command,
            context
        }
    }

    pub(crate) fn handle(&mut self, mut stream: &TcpStream, server_context: Arc<std::sync::Mutex<ServerContext>>) {
        let mut command = RedisCommand::create(self.command.clone(), self.context, server_context);
        let result = command.execute()
            .inspect_err(|err| eprintln!("{}", err))
            .unwrap_or_else(|e| e.to_string());

        stream.write_all(result.as_bytes()).unwrap();
    }
}
