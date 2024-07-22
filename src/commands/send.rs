use std::{
    fs::metadata,
    io::{self, Error, ErrorKind},
    path::PathBuf,
};

use tokio::net::TcpStream;

use crate::{
    comms::{send_meta_data, wait_for_receiver},
    models::SendMetaData,
    transmission::transfer_file_to_tcp,
    utils::{copy_key_to_clipbpard, get_key_from_conn},
};

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

    let meta_data = SendMetaData {
        file_name: file_path
            .file_name()
            .ok_or(Error::new(ErrorKind::Other, "Invalid file path"))?
            .to_str()
            .ok_or(Error::new(ErrorKind::Other, "Invalid file path"))?
            .to_string(),
        file_size: metadata(file_path)?.len(),
    };
    println!("Receiver info: {:?}", meta_data);
    send_meta_data(&mut connection, &meta_data).await?;

    let file_key = get_key_from_conn(&mut connection).await?;

    copy_key_to_clipbpard(file_key);

    wait_for_receiver(&mut connection).await?;

    transfer_file_to_tcp(file_path, &mut connection).await?;

    Ok(())
}
