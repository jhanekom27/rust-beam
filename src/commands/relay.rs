use std::{io, sync::Arc};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    select,
    sync::Mutex,
};

use crate::{
    utils::{get_key_from_conn, get_random_name},
    ReceiverInfo,
};
use crate::{Session, State};

async fn notify_sender(sender_lock: Arc<Mutex<TcpStream>>) -> io::Result<()> {
    sender_lock.lock().await.write_all(&[1]).await?;
    Ok(())
}

async fn get_receiver_info(
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

pub async fn relay(state: Arc<State>) -> io::Result<()> {
    println!("Relaying data");
    println!("{:?}", state);

    // Create sender and reciver listener as tcpListener
    let sender_listener = TcpListener::bind("0.0.0.0:7878").await?;
    let receiver_listener = TcpListener::bind("0.0.0.0:7879").await?;

    loop {
        select! {
            // Get the sender connection and add to state
            sender = sender_listener.accept() => {
                let (mut sender_conn, _) = sender?;
                // TODO: change to struct in the future
                let file_key = get_random_name();
                println!("{}", file_key);

                let receiver_info = get_receiver_info(&mut sender_conn).await?;
                println!("{:?}", receiver_info);

                sender_conn.write_all(file_key.as_bytes()).await?;

                state.sessions.lock().await.insert(
                    file_key.clone(),
                    Session {
                        sender_connection: Arc::new(Mutex::new(sender_conn)),
                        receiver_info: ReceiverInfo {
                            file_name: file_key}

                    },
                );
                println!("{:?}", state);
            }
            // Get the receiver connection
            receiver = receiver_listener.accept() => {
                println!("Receiver connected");
                let (mut receiver_conn, _) = receiver?;

                let file_key = get_key_from_conn(&mut receiver_conn).await?;
                println!("{}", file_key);

                let sender_conn = match state.sessions.lock().await.get(&file_key) {
                    Some(session) => session.sender_connection.clone(),
                    None => {
                        println!("No sender connection found for receiver");
                        continue
                    },
                };

                // Let the sender know the receiver is ready
                notify_sender(sender_conn.clone()).await?;

                tokio::spawn(async move {
                    let mut buffer = [0; 1024];
                    let mut sender_conn_guard = sender_conn.lock().await;

                    loop {
                        let n = match sender_conn_guard.read(&mut buffer).await {
                            Ok(n) if n == 0 => return io::Result::Ok(()),
                            Ok(n) => n,
                            Err(e) => return Err(e),
                        };
                        receiver_conn.write_all(&buffer[..n]).await?;
                    }

                });

                // Remove the stored session
                state.sessions.lock().await.remove(&file_key);
            }
        }
    }
}
