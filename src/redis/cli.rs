use clap::Parser;

#[derive(Parser)]
pub struct ServerArguments {
    #[arg(short, long, default_value = "6379")]
    pub port: u16,
    
    #[arg(short, long)]
    pub replicaof: Option<String>
}