use std::io::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub fn get_random_name() -> String {
    memorable_wordlist::kebab_case(30)
}

fn get_key_from_buf(buf: &[u8]) -> String {
    String::from_utf8(buf.to_vec())
        .expect("Invalid UTF-8 Sequence")
        .trim_end_matches("\0")
        .to_string()
}

pub async fn get_key_from_conn(conn: &mut TcpStream) -> Result<String, Error> {
    let buf = &mut [0; 32];
    conn.read(buf).await?;

    let file_key = get_key_from_buf(buf);
    // TODO: add in debug log

    Ok(file_key)
}
