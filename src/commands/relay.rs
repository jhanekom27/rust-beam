use std::{io, sync::Arc};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    select,
    sync::Mutex,
};

use crate::utils::{get_key_from_conn, get_random_name};
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
                // TODO: change to struct in the future
                let file_key = get_random_name();
                println!("{}", file_key);

                sender_conn.write_all(file_key.as_bytes()).await?;

                state.sessions.lock().await.insert(
                    file_key,
                    Session {
                        sender_connection: Arc::new(Mutex::new(sender_conn)),
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
