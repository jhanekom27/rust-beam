use std::io;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{comms::get_receiver_info, ReceiverInfo};

pub async fn receive_file(
    sender_key: &String,
    server_address: &str,
) -> io::Result<()> {
    println!("Receiving file with key: {}", sender_key);
    let mut connection = TcpStream::connect(server_address).await?;

    connection.write_all(sender_key.as_bytes()).await?;

    let receiver_info = get_receiver_info(&mut connection).await?;
    let ReceiverInfo { file_name } = receiver_info;

    // TODO: Allow overwriting filename from cli args
    let mut file = tokio::fs::File::create(file_name).await?;
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
