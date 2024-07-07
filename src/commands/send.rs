use std::io;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub async fn send_file(
    file_path: &str,
    server_address: &str,
) -> io::Result<()> {
    println!("Sending file: {}", file_path);
    let mut file = tokio::fs::File::open(file_path).await?;
    let mut buffer = [0; 1024];
    let mut connection = TcpStream::connect(server_address).await?;

    let file_key_buffer = &mut [0; 32];
    connection.read(file_key_buffer).await?;
    let file_key = String::from_utf8(file_key_buffer.to_vec())
        .expect("Invalid UTF-8 Sequence");

    println!("{:?}", file_key);

    while let Ok(n) = file.read(&mut buffer).await {
        if n == 0 {
            break;
        }
        connection.write_all(&buffer[..n]).await?;
    }
    Ok(())
}
