use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: [send|receive|relay] [FILE_PATH|ADDRESS:PORT]");
        return Ok(());
    }

    match args[1].as_str() {
        "send" if args.len() == 3 => {
            let filepath = &args[2];
            send_file(filepath).await?;
        }
        "receive" if args.len() == 3 => {
            let address = &args[2];
            println!("Address: {address}");
            receive_file(address).await?;
        }
        "relay" => {
            relay().await?;
        }
        _ => println!("Invalid usage"),
    }

    Ok(())
}

async fn send_file(filepath: &str) -> io::Result<()> {
    let mut connection = TcpStream::connect("127.0.0.1:7878").await?;
    let mut file = tokio::fs::File::open(filepath).await?;
    let mut buffer = [0; 1024];

    while let Ok(n) = file.read(&mut buffer).await {
        if n == 0 {
            break;
        }
        connection.write_all(&buffer[..n]).await?;
    }

    println!("File sent successfully");
    Ok(())
}

async fn receive_file(address: &str) -> io::Result<()> {
    let listener = TcpListener::bind(address).await?;
    println!("Listening on {}", address);
    let (mut connection, _) = listener.accept().await?;

    let mut file = tokio::fs::File::create("downloaded_file").await?;
    let mut buffer = [0; 1024];

    while let Ok(n) = connection.read(&mut buffer).await {
        if n == 0 {
            break;
        }
        file.write_all(&buffer[..n]).await?;
    }

    println!("File received successfully");
    Ok(())
}

async fn relay() -> io::Result<()> {
    let sender = TcpStream::connect("127.0.0.1:7878").await?;
    let receiver = TcpListener::bind("0.0.0.0:7879").await?;
    println!("Relay server running");

    let (mut receiver_conn, _) = receiver.accept().await?;
    let (mut sender_read, mut sender_write) = sender.into_split();

    // Relay from sender to receiver
    tokio::spawn(async move {
        io::copy(&mut sender_read, &mut receiver_conn)
            .await
            .expect("Failed to copy from sender to receiver");
    });

    Ok(())
}
