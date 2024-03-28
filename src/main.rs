use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    // println!("Hello, world!");
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        println!("Usage: p2p_file_share [send|receive] [FILE_PATH|ADDRESS:PORT]");
        return;
    }

    match args[1].as_str() {
        "send" if args.len() == 3 => {
            let filepath = &args[2];
            start_sender(filepath).await;
        }
        "receive" if args.len() == 3 => {
            let address = &args[2];
            println!("Address: {address}");
            start_receiver(address).await;
        }
        _ => println!("Invalid usage"),
    }
}

async fn start_sender(filepath: &str) {
    println!("Starting sender for file: {filepath}");

    let listener = TcpListener::bind("0.0.0.0:7878").await.unwrap();
    println!("Listening on port 7878...");

    match listener.accept().await {
        Ok((mut socket, _)) => {
            println!("Client connected");
            let mut file = tokio::fs::File::open(filepath).await.unwrap();
            let mut buffer = [0; 1024];

            loop {
                let n = file.read(&mut buffer).await.unwrap();
                if n == 0 {
                    break;
                }
                socket.write_all(&buffer[..n]).await.unwrap();
            }

            println!("File sent successfully");
        }
        Err(e) => println!("Failed to accept connection: {}", e),
    }
}

async fn start_receiver(address: &str) {
    let mut stream = TcpStream::connect(address).await.unwrap();
    println!("Connected to the server on address: {address}");

    let mut file = tokio::fs::File::create("downloaded_file").await.unwrap();
    let mut buffer = [0; 1024];

    loop {
        let n = stream.read(&mut buffer).await.unwrap();
        if n == 0 {
            break;
        }
        file.write_all(&buffer[..n]).await.unwrap();
    }

    println!("File downloaded successfully");
}
