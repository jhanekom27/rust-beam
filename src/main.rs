mod cli;
mod commands;
mod config;
mod utils;

use std::{collections::HashMap, io, sync::Arc};

use clap::Parser;
use tokio::{net::TcpStream, sync::Mutex};

use cli::{Cli, Commands};
use commands::receive::receive_file;
use commands::relay::relay;
use commands::send::send_file;
use config::get_config;

#[derive(Debug)]
struct State {
    sessions: Mutex<HashMap<String, Session>>,
}

#[derive(Debug)]
struct Session {
    sender_connection: Arc<Mutex<TcpStream>>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Cli::parse();
    println!("{:?}", args);

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
