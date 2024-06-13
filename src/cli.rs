use clap::{Args, Parser, Subcommand};
use uuid::Uuid;

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub commands: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Send(SendArgs),
    Receive(ReceiveArgs),
    Relay,
}

#[derive(Debug, Args)]
pub struct SendArgs {
    #[clap(short, long)]
    pub file_path: String,
}

#[derive(Debug, Args)]
pub struct ReceiveArgs {
    pub uuid: Uuid,
}
