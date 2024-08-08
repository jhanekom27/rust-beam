use std::{io, sync::Arc};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
    task,
};

use crate::{
    comms::{
        get_inbound, get_meta_data, notify_sender, send_meta_data,
        send_outbound,
    },
    models::{Session, State},
    utils::get_key_from_conn,
};

async fn handle_sender(
    mut sender_conn: TcpStream,
    state: Arc<State>,
) -> Result<(), io::Error> {
    let receiver_info = get_meta_data(&mut sender_conn).await?;
    println!("{:?}", receiver_info);

    let file_key = receiver_info.sender_key.clone();
    let sender_connection = Arc::new(Mutex::new(sender_conn));

    state.sessions.lock().await.insert(
        file_key,
        Session {
            sender_connection: sender_connection.clone(), // Clone the Arc for the session
            receiver_info,
        },
    );
    println!("{:?}", state);

    Ok(())
}

async fn handle_receiver(
    mut receiver_conn: TcpStream,
    state: Arc<State>,
) -> Result<(), io::Error> {
    println!("Receiver connected");

    let file_key = get_key_from_conn(&mut receiver_conn).await?;
    println!("{}", file_key);

    let mut locked_sessions = state.sessions.lock().await;
    let session = match locked_sessions.remove(&file_key) {
        Some(session) => session,
        None => {
            // TODO: send error message to receiver and handle by server
            println!("No sender connection found for receiver");
            return Ok(());
        }
    };

    let mut sender_conn_guard = session.sender_connection.lock().await;
    let receiver_info = &session.receiver_info;

    // Let the sender know the receiver is ready
    notify_sender(&mut sender_conn_guard).await?;
    send_meta_data(&mut receiver_conn, &receiver_info).await?;

    // TODO: do the key exchange for encryption
    let sender_message = get_inbound(&mut sender_conn_guard).await?;

    let receiver_message = get_inbound(&mut receiver_conn).await?;

    send_outbound(&mut sender_conn_guard, &receiver_message).await?;
    send_outbound(&mut receiver_conn, &sender_message).await?;

    let sender_conn_clone = session.sender_connection.clone();

    tokio::spawn(async move {
        let mut buffer = [0; 1024];
        let mut sender_conn_guard = sender_conn_clone.lock().await; // Lock the connection here

        loop {
            let n = match sender_conn_guard.read(&mut buffer).await {
                Ok(n) if n == 0 => return io::Result::Ok(()),
                Ok(n) => n,
                Err(e) => return Err(e),
            };
            receiver_conn.write_all(&buffer[..n]).await?;
        }
    });

    Ok(())
}

async fn handle_sender_listener(
    sender_listener: TcpListener,
    state: Arc<State>,
) {
    loop {
        match sender_listener.accept().await {
            Ok((sender_connection, _)) => {
                let state_clone = Arc::clone(&state);
                task::spawn(async move {
                    if let Err(e) =
                        handle_sender(sender_connection, state_clone).await
                    {
                        eprintln!("Error handling sender: {:?}", e);
                    }
                });
            }
            Err(e) => {
                println!("Error accepting sender connection: {:?}", e);
            }
        }
    }
}

async fn handle_receiver_listener(
    receiver_listener: TcpListener,
    state: Arc<State>,
) {
    loop {
        match receiver_listener.accept().await {
            Ok((receiver, _)) => {
                let state_clone = Arc::clone(&state);
                task::spawn(async move {
                    if let Err(e) = handle_receiver(receiver, state_clone).await
                    {
                        eprintln!("Error handling receiver: {:?}", e);
                    }
                });
            }
            Err(e) => {
                println!("Error accepting reciver connection: {:?}", e);
            }
        }
    }
}

pub async fn relay(state: Arc<State>) -> io::Result<()> {
    println!("Relaying data");

    // Create sender and reciver listener as tcpListener
    let sender_listener = TcpListener::bind("0.0.0.0:7878").await?;
    let receiver_listener = TcpListener::bind("0.0.0.0:7879").await?;

    let sender_task = {
        let state_clone = state.clone();
        task::spawn(handle_sender_listener(sender_listener, state_clone))
    };
    let receiver_task = {
        let state_clone = state.clone();
        task::spawn(handle_receiver_listener(receiver_listener, state_clone))
    };

    let _ = tokio::join!(sender_task, receiver_task);
    Ok(())
}
