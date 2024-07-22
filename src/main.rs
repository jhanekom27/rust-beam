include!(concat!(env!("OUT_DIR"), "/password.rs"));

mod cli;
mod commands;
mod comms;
mod config;
mod models;
mod transmission;
mod ui;
mod utils;

use std::{collections::HashMap, io, sync::Arc};

use clap::Parser;
use models::State;
use tokio::sync::Mutex;

use cli::{Cli, Commands};
use commands::receive::receive_file;
use commands::relay::relay;
use commands::send::send_file;
use config::get_config;

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Cli::parse();
    println!("{:?}", args);
    println!("The compiled password is: {}", PASSWORD);

    let config = get_config();
    let send_server_address =
        format!("{}:{}", config.server_url, config.send_port);
    let receive_server_address =
        format!("{}:{}", config.server_url, config.receive_port);

    let state = Arc::new(State {
        sessions: Mutex::new(HashMap::new()),
    });

    match args.commands {
        Commands::Send(send_args) => {
            send_file(&send_args.file_path, &send_server_address).await?;
        }
        Commands::Receive(receive_args) => {
            receive_file(&receive_args.sender_key, &receive_server_address)
                .await?;
        }
        Commands::Relay => {
            relay(state.clone()).await?;
        }
    }

    println!("Success");
    Ok(())
}
