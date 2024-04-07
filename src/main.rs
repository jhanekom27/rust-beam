// use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
// use tokio::net::{TcpListener, TcpStream};

use std::{collections::HashMap, io, sync::Arc};

use clap::{Args, Parser, Subcommand};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    select,
    sync::Mutex,
};
use uuid::Uuid;

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Send(SendArgs),
    Receive(ReceiveArgs),
    Relay,
}

#[derive(Debug, Args)]
struct SendArgs {
    #[clap(short, long)]
    file_path: String,
}

#[derive(Debug, Args)]
struct ReceiveArgs {
    uuid: Uuid,
}

#[derive(Debug)]
struct State {
    sessions: Mutex<HashMap<Uuid, Session>>,
}

#[derive(Debug)]
struct Session {
    sender_connection: Arc<Mutex<TcpStream>>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Cli::parse();
    println!("{:?}", args);

    let state = Arc::new(State {
        sessions: Mutex::new(HashMap::new()),
    });

    match args.commands {
        Commands::Send(send_args) => {
            send_file(&send_args.file_path).await?;
        }
        Commands::Receive(receive_args) => {
            receive_file(receive_args.uuid).await?;
        }
        Commands::Relay => {
            relay(state.clone()).await?;
        }
    }

    println!("Success");
    Ok(())
}

async fn send_file(file_path: &str) -> io::Result<()> {
    println!("Sending file: {}", file_path);
    let mut file = tokio::fs::File::open(file_path).await?;
    let mut buffer = [0; 1024];
    let mut connection = TcpStream::connect("0.0.0.0:7878").await?;

    while let Ok(n) = file.read(&mut buffer).await {
        if n == 0 {
            break;
        }
        connection.write_all(&buffer[..n]).await?;
    }
    Ok(())
}

async fn receive_file(uuid: Uuid) -> io::Result<()> {
    println!("Receiving file with UUID: {}", uuid);
    let mut file = tokio::fs::File::create(uuid.to_string()).await?;
    let mut buffer = [0; 1024];
    let mut connection = TcpStream::connect("0.0.0.0:7879").await?;

    connection.write_all(uuid.as_bytes()).await?;

    while let Ok(n) = connection.read(&mut buffer).await {
        if n == 0 {
            break;
        }
        file.write_all(&buffer[..n]).await?;
    }

    println!("File received successfully");

    Ok(())
}

async fn relay(state: Arc<State>) -> io::Result<()> {
    println!("Relaying data");
    println!("{:?}", state);

    // Create sender and reciver listener as tcpListener
    let sender_listener = TcpListener::bind("0.0.0.0:7878").await?;
    let receiver_listener = TcpListener::bind("0.0.0.0:7879").await?;

    loop {
        select! {
            // Get the sender connection and add to state
            sender = sender_listener.accept() => {
                let (sender_conn, _) = sender?;
                let uuid = Uuid::new_v4();
                println!("{:?}", uuid);
                state.sessions.lock().await.insert(
                    uuid,
                    Session {
                        sender_connection: Arc::new(Mutex::new(sender_conn)),
                    },
                );
                println!("{:?}", state);
            }
            // Get the receiver connection
            receiver = receiver_listener.accept() => {
                let (mut receiver_conn, _) = receiver?;
                let uuid_buf = &mut [0; 16];
                receiver_conn.read(uuid_buf).await?;
                let receiver_uuid = Uuid::from_bytes(*uuid_buf);
                println!("{:?}", receiver_uuid);

                let sender_conn = state.sessions.lock().await.get(&receiver_uuid).unwrap().sender_connection.clone();


                let sender_to_receiver = tokio::spawn(async move {
                    let mut buffer = [0; 1024];
                    let mut sender_conn_guard = sender_conn.lock().await;

                    loop {
                        let n = match sender_conn_guard.read(&mut buffer).await {
                            Ok(n) if n == 0 => return io::Result::Ok(()),
                            Ok(n) => n,
                            Err(e) => return Err(e),
                        };
                        receiver_conn.write_all(&buffer[..n]).await?;
                    }

                });

                let _ = sender_to_receiver.await;
            }
        }
    }
}
