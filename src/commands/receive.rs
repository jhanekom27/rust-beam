use std::{io, path::PathBuf};

use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::{
    comms::get_receiver_info, models::ReceiverInfo,
    transmission::transfer_tcp_to_file,
};

pub async fn receive_file(
    sender_key: &String,
    server_address: &str,
) -> io::Result<()> {
    println!("Receiving file with key: {}", sender_key);
    let mut connection = TcpStream::connect(server_address).await?;

    connection.write_all(sender_key.as_bytes()).await?;

    let ReceiverInfo {
        file_name,
        file_size,
    } = get_receiver_info(&mut connection).await?;

    transfer_tcp_to_file(&PathBuf::from(file_name), &mut connection, file_size)
        .await?;

    Ok(())
}
