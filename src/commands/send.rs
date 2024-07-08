use std::{
    io::{self, Error, ErrorKind},
    path::PathBuf,
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{
    utils::{copy_key_to_clipbpard, get_key_from_conn},
    ReceiverInfo,
};

async fn wait_for_receiver(mut connection: &mut TcpStream) -> io::Result<()> {
    println!("Waiting for receiver to be ready");
    let receiver_status = &mut [0; 1];
    connection.read(receiver_status).await?;
    println!("Receiver is ready");
    Ok(())
}

async fn send_receiver_info(
    mut connection: &mut TcpStream,
    receiver_info: &ReceiverInfo,
) -> io::Result<()> {
    let receiver_info_json = serde_json::to_string(receiver_info)?;
    connection.write_all(receiver_info_json.as_bytes()).await?;

    Ok(())
}

pub async fn send_file(
    file_path: &PathBuf,
    server_address: &str,
) -> io::Result<()> {
    println!(
        "Sending file: {}",
        file_path
            .to_str()
            .ok_or(io::Error::new(io::ErrorKind::Other, "Invalid file path"))?
    );

    let mut connection = TcpStream::connect(server_address).await?;

    let receiver_info = ReceiverInfo {
        file_name: file_path
            .file_name()
            .ok_or(Error::new(ErrorKind::Other, "Invalid file path"))?
            .to_str()
            .ok_or(Error::new(ErrorKind::Other, "Invalid file path"))?
            .to_string(),
    };
    println!("Receiver info: {:?}", receiver_info);
    send_receiver_info(&mut connection, &receiver_info).await?;

    let file_key = get_key_from_conn(&mut connection).await?;

    copy_key_to_clipbpard(file_key);

    wait_for_receiver(&mut connection).await?;

    let mut file = tokio::fs::File::open(file_path).await?;
    let mut file_buffer = [0; 1024];
    while let Ok(n) = file.read(&mut file_buffer).await {
        if n == 0 {
            break;
        }
        connection.write_all(&file_buffer[..n]).await?;
    }
    Ok(())
}
