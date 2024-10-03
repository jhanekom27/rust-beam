use copypasta::{ClipboardContext, ClipboardProvider};
use crossterm::{
    cursor::MoveToNextLine,
    event::{self, Event, KeyCode},
    execute,
    style::{
        Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor,
    },
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io::{self, Error};

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
    let mut stdout = io::stdout();

    // Enable raw mode
    enable_raw_mode().unwrap();

    // Output formatted text to the terminal
    execute!(
        stdout,
        Print("copy key to clipboard: "),
        SetAttribute(Attribute::Bold),
        SetForegroundColor(Color::Green),
        Print(file_key.clone()),
        ResetColor,
        SetAttribute(Attribute::Reset),
        MoveToNextLine(1)
    )
    .unwrap();

    // Listen for key presses
    loop {
        // Wait for an event (blocking)
        if let Event::Key(key_event) = event::read().unwrap() {
            match key_event.code {
                KeyCode::Char(' ') => {
                    let mut ctx: ClipboardContext =
                        ClipboardContext::new().unwrap();
                    ctx.set_contents(file_key.clone()).unwrap();
                    break;
                }
                _ => {}
            }
        }
    }

    // Return stdout to normal mode (disable raw mode)
    disable_raw_mode().unwrap();
}
