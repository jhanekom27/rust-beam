use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

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
    pub file_path: PathBuf,
}

#[derive(Debug, Args)]
pub struct ReceiveArgs {
    pub sender_key: String,
}
