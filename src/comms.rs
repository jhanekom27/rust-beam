use std::{io, sync::Arc};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};

use crate::models::ReceiverInfo;

pub async fn wait_for_receiver(connection: &mut TcpStream) -> io::Result<()> {
    println!("Waiting for receiver to be ready");
    let receiver_status = &mut [0; 1];
    connection.read(receiver_status).await?;
    println!("Receiver is ready");
    Ok(())
}

pub async fn notify_sender(
    sender_lock: Arc<Mutex<TcpStream>>,
) -> io::Result<()> {
    sender_lock.lock().await.write_all(&[1]).await?;
    Ok(())
}

pub async fn send_receiver_info(
    connection: &mut TcpStream,
    receiver_info: &ReceiverInfo,
) -> io::Result<()> {
    println!("Sending receiver info: {:?}", receiver_info);
    let receiver_info_json = serde_json::to_string(receiver_info)?;
    connection.write_all(receiver_info_json.as_bytes()).await?;

    Ok(())
}

pub async fn get_receiver_info(
    connection: &mut TcpStream,
) -> io::Result<ReceiverInfo> {
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
            if let Ok(my_struct) = serde_json::from_str::<ReceiverInfo>(
                std::str::from_utf8(received).unwrap(),
            ) {
                println!("Received: {:?}", my_struct);
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
