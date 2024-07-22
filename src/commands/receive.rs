use std::{io, path::PathBuf};

use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::{
    comms::get_meta_data, models::SendMetaData,
    transmission::transfer_tcp_to_file,
};

pub async fn receive_file(
    sender_key: &String,
    server_address: &str,
) -> io::Result<()> {
    println!("Receiving file with key: {}", sender_key);
    let mut connection = TcpStream::connect(server_address).await?;

    connection.write_all(sender_key.as_bytes()).await?;

    let SendMetaData {
        file_name,
        file_size,
    } = get_meta_data(&mut connection).await?;

    transfer_tcp_to_file(&PathBuf::from(file_name), &mut connection, file_size)
        .await?;

    Ok(())
}
