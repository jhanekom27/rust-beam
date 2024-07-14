use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::prelude::*;

use std::{
    fs::metadata,
    io::{Error, ErrorKind},
    path::PathBuf,
};
use tokio::fs::File;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

use crate::models::ReceiverInfo;

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

    // let mut connection = TcpStream::connect(server_address).await?;

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
    // send_receiver_info(&mut connection, &receiver_info).await?;

    // let file_key = get_key_from_conn(&mut connection).await?;

    // copy_key_to_clipbpard(file_key);

    // wait_for_receiver(&mut connection).await?;

    // transfer_file_to_tcp(file_path, &mut connection).await?;

    let output_path = "foo.gz";

    // Read the input file
    let mut input_file = File::open(file_path).await?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer).await?;

    // Compress the data using GzEncoder into an in-memory buffer
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&buffer)?;
    let compressed_data = encoder.finish()?;

    // Write the compressed data to the output file asynchronously
    let mut output_file = File::create(output_path).await?;
    output_file.write_all(&compressed_data).await?;
    output_file.flush().await?;

    Ok(())
}
