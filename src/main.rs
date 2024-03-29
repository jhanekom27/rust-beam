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
    let (tx, mut rx) = mpsc::channel(1);
    let sender_listener = TcpListener::bind("0.0.0.0:7878").await?;
    let receiver_listener = TcpListener::bind("0.0.0.0:7879").await?;

    println!("Relay server running");

    tokio::spawn(async move {
        while let Ok((sender_conn, _)) = sender_listener.accept().await {
            let tx = tx.clone();
            tokio::spawn(async move {
                tx.send(sender_conn)
                    .await
                    .expect("Failed to send sender connection");
            });
        }
    });

    while let Ok((mut receiver_conn, _)) = receiver_listener.accept().await {
        if let Some(sender_conn) = rx.recv().await {
            let (mut sender_read, mut sender_write) = sender_conn.into_split();
            let (mut receiver_read, mut receiver_write) = receiver_conn.into_split();

            tokio::spawn(async move {
                io::copy(&mut sender_read, &mut receiver_write)
                    .await
                    .expect("Failed to relay data from sender to receiver");
            });

            tokio::spawn(async move {
                io::copy(&mut receiver_read, &mut sender_write)
                    .await
                    .expect("Failed to relay data from receiver to sender");
            });
        }
    }

    Ok(())
}
