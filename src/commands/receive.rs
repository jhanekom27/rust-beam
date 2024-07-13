use std::io;

use indicatif::{ProgressBar, ProgressStyle};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{comms::get_receiver_info, models::ReceiverInfo};

pub async fn receive_file(
    sender_key: &String,
    server_address: &str,
) -> io::Result<()> {
    println!("Receiving file with key: {}", sender_key);
    let mut connection = TcpStream::connect(server_address).await?;

    connection.write_all(sender_key.as_bytes()).await?;

    let receiver_info = get_receiver_info(&mut connection).await?;
    let ReceiverInfo {
        file_name,
        file_size,
    } = receiver_info;

    // Create a progress bar
    let progress_bar = ProgressBar::new(file_size);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})").unwrap()
    );

    // TODO: Allow overwriting filename from cli args
    let mut file = tokio::fs::File::create(file_name).await?;
    let mut buffer = [0; 1024];
    let mut bytes_read = 0;

    while let Ok(n) = connection.read(&mut buffer).await {
        if n == 0 {
            break;
        }
        file.write_all(&buffer[..n]).await?;
        // TODO: add update callback
        bytes_read += n as u64;
        progress_bar.set_position(bytes_read);
    }

    progress_bar.finish_with_message("Done");
    println!("File received successfully");

    Ok(())
}
