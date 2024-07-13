use std::{
    fs::metadata,
    io::{self, Error, ErrorKind},
    path::PathBuf,
};

use indicatif::{ProgressBar, ProgressStyle};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{
    comms::{send_receiver_info, wait_for_receiver},
    models::ReceiverInfo,
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

    let receiver_info = ReceiverInfo {
        file_name: file_path
            .file_name()
            .ok_or(Error::new(ErrorKind::Other, "Invalid file path"))?
            .to_str()
            .ok_or(Error::new(ErrorKind::Other, "Invalid file path"))?
            .to_string(),
        file_size: metadata(file_path)?.len(),
    };
    println!("Receiver info: {:?}", receiver_info);
    send_receiver_info(&mut connection, &receiver_info).await?;

    let file_key = get_key_from_conn(&mut connection).await?;

    copy_key_to_clipbpard(file_key);

    wait_for_receiver(&mut connection).await?;

    // Create a progress bar
    let progress_bar = ProgressBar::new(receiver_info.file_size);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})").unwrap()
    );

    let mut file = tokio::fs::File::open(file_path).await?;
    let mut file_buffer = [0; 1024];
    let mut bytes_read = 0;
    // TODO: extract this to a function
    while let Ok(n) = file.read(&mut file_buffer).await {
        if n == 0 {
            break;
        }
        connection.write_all(&file_buffer[..n]).await?;
        // TODO: add update callback
        bytes_read += n as u64;
        progress_bar.set_position(bytes_read);
    }

    progress_bar.finish_with_message("done");
    Ok(())
}
