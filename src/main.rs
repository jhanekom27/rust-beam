use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

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
        "receive" if args.len() == 2 => {
            // let address = &args[2];
            // println!("Address: {address}");
            receive_file("127.0.0.1:7879").await?;
        }
        "relay" => {
            relay().await?;
        }
        _ => println!("Invalid usage"),
    }

    Ok(())
}

async fn send_file(ilepath: &str) -> io::Result<()> {
    let mut file = tokio::fs::File::open(ilepath).await?;
    let mut buffer = [0; 1024];
    let mut connection = TcpStream::connect("127.0.0.1:7878").await?;

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
    let mut connection = TcpStream::connect(address).await?;
    let mut file = tokio::fs::File::create("received_file").await?;
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
    let sender_listener = TcpListener::bind("0.0.0.0:7878").await?;
    let receiver_listener = TcpListener::bind("0.0.0.0:7879").await?;
    println!("Relay server running");

    loop {
        // Wait for the sender to connect
        let (mut sender_conn, _) = match sender_listener.accept().await {
            Ok(conn) => conn,
            Err(e) => {
                eprintln!("Error accepting sender connection: {}", e);
                continue;
            }
        };
        println!("Sender connected");

        // Wait for the receiver to connect
        let (mut receiver_conn, _) = match receiver_listener.accept().await {
            Ok(conn) => conn,
            Err(e) => {
                eprintln!("Error accepting receiver connection: {}", e);
                continue;
            }
        };
        println!("Receiver connected");

        // Relay data from sender to receiver
        let sender_to_receiver = tokio::spawn(async move {
            let mut buffer = [0; 1024];
            loop {
                let n = match sender_conn.read(&mut buffer).await {
                    Ok(n) if n == 0 => return io::Result::Ok(()),
                    Ok(n) => n,
                    Err(e) => return Err(e),
                };
                receiver_conn.write_all(&buffer[..n]).await?;
            }
        });

        // Wait for the relaying task to finish
        let _ = sender_to_receiver.await;
    }
}
