use clap::Parser;

#[derive(Parser)]
pub struct Arguments {
    #[arg(short, long, default_value = "6379")]
    pub port: String,
    
    #[arg(short, long)]
    pub replicaof: Option<String>
}