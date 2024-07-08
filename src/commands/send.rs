use std::io;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::utils::get_key_from_buf;

pub async fn send_file(
    file_path: &str,
    server_address: &str,
) -> io::Result<()> {
    println!("Sending file: {}", file_path);
    let mut file = tokio::fs::File::open(file_path).await?;
    let mut file_buffer = [0; 1024];
    let mut connection = TcpStream::connect(server_address).await?;

    let file_key_buffer = &mut [0; 32];
    connection.read(file_key_buffer).await?;

    let file_key = get_key_from_buf(file_key_buffer);

    println!("{}", file_key);

    while let Ok(n) = file.read(&mut file_buffer).await {
        if n == 0 {
            break;
        }
        connection.write_all(&file_buffer[..n]).await?;
    }
    Ok(())
}
