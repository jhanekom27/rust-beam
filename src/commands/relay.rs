use std::{io, sync::Arc};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
    task,
};

use crate::{
    comms::{get_inbound, get_meta_data, notify_sender, send_meta_data},
    models::{Session, State},
    utils::get_key_from_conn,
};

async fn handle_sender(
    mut sender_conn: TcpStream,
    state: Arc<State>,
) -> Result<(), io::Error> {
    // Handle the specific sender connection
    println!("New sender connected: {:?}", sender_conn.peer_addr());
    // Implement your logic to read from the sender
    let receiver_info = get_meta_data(&mut sender_conn).await?;
    println!("{:?}", receiver_info);
    let file_key = receiver_info.sender_key.clone();

    let sender_connection = Arc::new(Mutex::new(sender_conn));

    println!("locking state");
    state.sessions.lock().await.insert(
        file_key.clone(),
        Session {
            sender_connection: sender_connection.clone(), // Clone the Arc for the session
            receiver_info,
        },
    );
    println!("{:?}", state);
    println!("state unlocked");

    // receive the inbound message
    // let inbound_msg = get_inbound(&mut sender_conn).await?;
    // println!("locking connection");
    // let conn_clone = sender_connection.clone();
    // let mut conn = conn_clone.lock().await;
    // println!("Locked connection");
    // let inbound_msg = {
    //     // let mut conn = sender_connection.lock().await; // Lock the connection for use

    //     get_inbound(&mut conn).await? // Dereference the lock to get the underlying connection
    // };
    // println!("connection unlocked");
    // println!("Inbound message: {:?}", inbound_msg);
    Ok(())
}

async fn handle_receiver(
    mut receiver_conn: TcpStream,
    state: Arc<State>,
) -> Result<(), io::Error> {
    println!("Receiver connected");

    let file_key = get_key_from_conn(&mut receiver_conn).await?;
    println!("{}", file_key);

    println!("locking state");
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

    // TODO: do the key exchange for encryption
    let sender_message = get_inbound(&mut sender_conn_guard).await?;
    println!("Sender message: {:?}", sender_message);

    send_meta_data(&mut receiver_conn, &receiver_info).await?;

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
            Ok((sender, _)) => {
                let state_clone = Arc::clone(&state);
                task::spawn(async move {
                    if let Err(e) = handle_sender(sender, state_clone).await {
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

pub async fn relay(state: Arc<State>) -> io::Result<()> {
    println!("Relaying data");
    println!("{:?}", state);

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
