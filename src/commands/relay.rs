use std::{io, sync::Arc};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    select,
    sync::Mutex,
};

use crate::{
    comms::{get_meta_data, notify_sender, send_meta_data},
    models::{Session, State},
    utils::get_key_from_conn,
};

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


                let receiver_info = get_meta_data(&mut sender_conn).await?;
                println!("{:?}", receiver_info);
                let file_key = receiver_info.sender_key.clone();

                state.sessions.lock().await.insert(
                    file_key.clone(),
                    Session {
                        sender_connection: Arc::new(Mutex::new(sender_conn)),
                        receiver_info

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

                let mut locked_sessions = state.sessions.lock().await;
                let session = match locked_sessions.get(&file_key) {
                    Some(session) => session,
                    None => {
                        println!("No sender connection found for receiver");
                        continue
                    },
                };

                let sender_conn = session.sender_connection.clone();
                let receiver_info = &session.receiver_info;

                // Let the sender know the receiver is ready
                notify_sender(sender_conn.clone()).await?;

                // TODO: do the key exchange for encryption

                send_meta_data(&mut receiver_conn, &receiver_info).await?;

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
                locked_sessions.remove(&file_key);
            }
        }
    }
}
