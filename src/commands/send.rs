use std::io;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use uuid::Uuid;

pub async fn send_file(file_path: &str) -> io::Result<()> {
    println!("Sending file: {}", file_path);
    let mut file = tokio::fs::File::open(file_path).await?;
    let mut buffer = [0; 1024];
    let mut connection = TcpStream::connect("170.64.168.50:7878").await?;

    let uuid_buf = &mut [0; 16];
    connection.read(uuid_buf).await?;
    let relay_uuid = Uuid::from_bytes(*uuid_buf);

    println!("{:?}", relay_uuid);

    while let Ok(n) = file.read(&mut buffer).await {
        if n == 0 {
            break;
        }
        connection.write_all(&buffer[..n]).await?;
    }
    Ok(())
}
