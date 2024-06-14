use std::{io, sync::Arc};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    select,
    sync::Mutex,
};
use uuid::Uuid;

use crate::{Session, State};

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
                let uuid = Uuid::new_v4();
                println!("{:?}", uuid);

                sender_conn.write_all(uuid.as_bytes()).await?;


                state.sessions.lock().await.insert(
                    uuid,
                    Session {
                        sender_connection: Arc::new(Mutex::new(sender_conn)),
                    },
                );
                println!("{:?}", state);
            }
            // Get the receiver connection
            receiver = receiver_listener.accept() => {
                let (mut receiver_conn, _) = receiver?;
                let uuid_buf = &mut [0; 16];
                receiver_conn.read(uuid_buf).await?;
                let receiver_uuid = Uuid::from_bytes(*uuid_buf);
                println!("{:?}", receiver_uuid);


                let sender_conn = match state.sessions.lock().await.get(&receiver_uuid) {
                    Some(session) => session.sender_connection.clone(),
                    None => {
                        println!("No sender connection found for receiver");
                        continue
                    },
                };


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
                state.sessions.lock().await.remove(&receiver_uuid);
            }
        }
    }
}
