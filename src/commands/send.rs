include!(concat!(env!("OUT_DIR"), "/password.rs"));
use std::{
    fs::metadata,
    io::{self, Error, ErrorKind},
    path::PathBuf,
};

use tokio::net::TcpStream;

use crate::{
    comms::{
        get_inbound, send_meta_data, send_outbound, wait_for_receiver,
        SpakeMessage,
    },
    models::SendMetaData,
    transmission::transfer_file_to_tcp,
    utils::{copy_key_to_clipbpard, get_random_name},
};
use spake2::{Ed25519Group, Identity, Password, Spake2};

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
        sender_key: get_random_name(), // TODO: add CLI option here
    };
    println!("Receiver info: {:?}", meta_data);
    send_meta_data(&mut connection, &meta_data).await?;

    copy_key_to_clipbpard(meta_data.sender_key);

    wait_for_receiver(&mut connection).await?;

    // do the key exchange thingo
    let (spake, outbound_msg) = Spake2::<Ed25519Group>::start_a(
        &Password::new(PASSWORD),
        &Identity::new(b"sender"),
        &Identity::new(b"receiver"),
    );
    println!("outbound_msg: {:?}", outbound_msg);

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
    println!("Inbound message: {:?}", inbound_spake_message);

    // create the key
    let key1 = spake.finish(&inbound_spake_message.message).unwrap();
    println!("Key1: {:?}", key1);

    transfer_file_to_tcp(file_path, &mut connection).await?;

    Ok(())
}

// fn foo() {
//     // Both Alice and Bob share the same password.
//     let password = Password::new("password123");

//     // Initialize SPAKE2 instances for Alice and Bob.
//     let alice =
//         SPAKE2::<Ed25519Group>::new(Identity::new("alice"), password.clone());
//     let bob =
//         SPAKE2::<Ed25519Group>::new(Identity::new("bob"), password.clone());

//     // Alice and Bob generate their first messages to send to each other.
//     let alice_msg = alice.start();
//     let bob_msg = bob.start();

//     // Alice and Bob process each other's messages to generate their session keys.
//     let alice_key = alice.finish(bob_msg).unwrap();
//     let bob_key = bob.finish(alice_msg).unwrap();

//     // Verify that both session keys are equal.
//     assert_eq!(alice_key, bob_key);

//     println!("Alice and Bob have successfully established a shared key!");
// }
