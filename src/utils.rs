use copypasta::{ClipboardContext, ClipboardProvider};
use std::io;
use std::io::Error;
use std::io::Write;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{color, style};
use tokio::{io::AsyncReadExt, net::TcpStream};

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

pub fn copy_key_to_clipbpard(file_key: String) {
    let mut stdout = io::stdout().into_raw_mode().unwrap();
    let stdin = io::stdin();

    writeln!(
        stdout,
        "{}<space>{} copy key to clipboard: {}{}{}{}\r",
        style::Bold,
        style::Reset,
        color::Fg(color::Green),
        style::Bold,
        file_key,
        style::Reset
    )
    .unwrap();

    for c in stdin.keys() {
        match c.unwrap() {
            Key::Char(' ') => {
                let mut ctx: ClipboardContext =
                    ClipboardContext::new().unwrap();
                ctx.set_contents(file_key.clone()).unwrap();
                break;
            }
            _ => {}
        }
    }

    // return stdout to normal
    let _ = stdout.suspend_raw_mode();
}
