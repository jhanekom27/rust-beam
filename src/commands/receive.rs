use std::io;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use uuid::Uuid;

pub async fn receive_file(uuid: Uuid) -> io::Result<()> {
    println!("Receiving file with UUID: {}", uuid);
    let mut file = tokio::fs::File::create(uuid.to_string()).await?;
    let mut buffer = [0; 1024];
    let mut connection = TcpStream::connect("170.64.168.50:7879").await?;

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
