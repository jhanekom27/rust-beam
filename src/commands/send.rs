use std::io;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::utils::{copy_key_to_clipbpard, get_key_from_conn};

pub async fn send_file(
    file_path: &str,
    server_address: &str,
) -> io::Result<()> {
    println!("Sending file: {}", file_path);
    let mut file = tokio::fs::File::open(file_path).await?;
    let mut file_buffer = [0; 1024];
    let mut connection = TcpStream::connect(server_address).await?;

    let file_key = get_key_from_conn(&mut connection).await?;

    copy_key_to_clipbpard(file_key);

    while let Ok(n) = file.read(&mut file_buffer).await {
        if n == 0 {
            break;
        }
        connection.write_all(&file_buffer[..n]).await?;
    }
    Ok(())
}
