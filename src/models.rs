use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::{net::TcpStream, sync::Mutex};

#[derive(Debug)]
pub struct State {
    pub sessions: Mutex<HashMap<String, Session>>,
}

#[derive(Debug)]
pub struct Session {
    pub sender_connection: Arc<Mutex<TcpStream>>,
    pub receiver_info: SendMetaData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SendMetaData {
    pub file_name: String,
    pub file_size: u64,
}
