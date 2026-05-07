use clap::Parser;

#[derive(Parser, Clone)]
pub struct Arguments {
    #[arg(short, long, default_value = "6379")]
    pub port: String,
    
    #[arg(short, long)]
    pub replicaof: Option<String>
}