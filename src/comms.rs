use std::io;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::models::SendMetaData;

pub async fn wait_for_receiver(connection: &mut TcpStream) -> io::Result<()> {
    println!("Waiting for receiver to be ready");
    let receiver_status = &mut [0; 1];
    connection.read(receiver_status).await?;
    println!("Receiver is ready");
    Ok(())
}

pub async fn notify_sender(connection: &mut TcpStream) -> io::Result<()> {
    connection.write_all(&[1]).await?;
    Ok(())
}

pub async fn send_meta_data(
    connection: &mut TcpStream,
    receiver_info: &SendMetaData,
) -> io::Result<()> {
    // TODO: create a generic function that can just send anr recieive structs as json
    let receiver_info_json = serde_json::to_string(receiver_info)?;
    connection.write_all(receiver_info_json.as_bytes()).await?;

    Ok(())
}
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct SpakeMessage {
    pub message: Vec<u8>,
}

pub async fn send_outbound(
    connection: &mut TcpStream,
    spake_message: &SpakeMessage,
) -> io::Result<()> {
    let spake_message_json = serde_json::to_string(spake_message)?;
    connection.write_all(spake_message_json.as_bytes()).await?;

    Ok(())
}

pub async fn get_inbound(
    connection: &mut TcpStream,
) -> io::Result<SpakeMessage> {
    let mut buffer = [0; 1024];
    match connection.read(&mut buffer).await {
        Ok(n) => {
            if n == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "No data received",
                ));
            }

            let received = &buffer[..n];
            if let Ok(my_struct) = serde_json::from_str::<SpakeMessage>(
                std::str::from_utf8(received).unwrap(),
            ) {
                return Ok(my_struct);
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to deserialize",
                ));
            }
        }
        Err(e) => return Err(e),
    }
}

pub async fn get_meta_data(
    connection: &mut TcpStream,
) -> io::Result<SendMetaData> {
    let mut buffer = [0; 1024];
    match connection.read(&mut buffer).await {
        Ok(n) => {
            if n == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "No data received",
                ));
            }

            let received = &buffer[..n];
            if let Ok(my_struct) = serde_json::from_str::<SendMetaData>(
                std::str::from_utf8(received).unwrap(),
            ) {
                return Ok(my_struct);
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to deserialize",
                ));
            }
        }
        Err(e) => return Err(e),
    }
}
