include!(concat!(env!("OUT_DIR"), "/password.rs"));
use std::{io, path::PathBuf};

use spake2::{Ed25519Group, Identity, Password, Spake2};
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::{
    comms::{get_inbound, get_meta_data, send_outbound, SpakeMessage},
    models::SendMetaData,
    transmission::transfer_tcp_to_file,
};

//TODO: maybe refactor to pass whole receiver args struct
pub async fn receive_file(
    sender_key: &String,
    server_address: &str,
    output_path: &Option<PathBuf>,
) -> io::Result<()> {
    println!("Receiving file with key: {}", sender_key);
    let mut connection = TcpStream::connect(server_address).await?;

    connection.write_all(sender_key.as_bytes()).await?;

    let SendMetaData {
        file_name, // TODO: also change to PathBuf
        file_size,
        sender_key: _,
    } = get_meta_data(&mut connection).await?;

    // do the key exchange thingo
    let (spake, outbound_msg) = Spake2::<Ed25519Group>::start_b(
        &Password::new(PASSWORD),
        &Identity::new(b"sender"),
        &Identity::new(b"receiver"),
    );

    // send the outbound message
    send_outbound(
        &mut connection,
        &SpakeMessage {
            message: outbound_msg,
        },
    )
    .await?;

    // receive the inbound message
    let inbound_spake_message = get_inbound(&mut connection).await?;

    // create the key
    let key2 = spake.finish(&inbound_spake_message.message).unwrap();

    // Unwrap the output path if provided otherwise use filename
    let save_path = match output_path {
        Some(path) => &path,
        _ => &PathBuf::from(file_name),
    };

    transfer_tcp_to_file(save_path, &mut connection, file_size, &key2).await?;

    Ok(())
}
