use std::io;
use std::pin::Pin;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::ui::ProgressBarTracker;

// constant buffer size
const BUFFER_SIZE: usize = 1024;

pub trait UpdateProgress {
    fn update_progress(&mut self, bytes_read: u64);
}

// Enum for mode to be encrypt or decrypt
pub enum Mode {
    Encrypt,
    Decrypt,
}

async fn transfer_bytes_from_source_to_sink(
    buffer: &mut [u8],
    source: &mut (dyn tokio::io::AsyncRead + Unpin),
    sink: &mut (dyn tokio::io::AsyncWrite + Unpin),
    progress_tracker: &mut dyn UpdateProgress,
    key: &[u8],
    mode: Mode,
) -> io::Result<()> {
    let mut bytes_read = 0;

    let mut pinned_source = Pin::new(source);
    let mut pinned_sink = Pin::new(sink);

    while let Ok(n) = pinned_source.as_mut().read(buffer).await {
        if n == 0 {
            break;
        }
        pinned_sink.as_mut().write_all(&buffer[..n]).await?;
        bytes_read += n;
        progress_tracker.update_progress(bytes_read as u64);
    }

    Ok(())
}

pub async fn transfer_file_to_tcp(
    file_path: &std::path::PathBuf,
    connection: &mut tokio::net::TcpStream,
    key: &[u8],
) -> io::Result<()> {
    let mut file = tokio::fs::File::open(file_path).await?;
    let mut buffer = [0; BUFFER_SIZE];
    let mut progress_tracker =
        ProgressBarTracker::new(file.metadata().await?.len());

    transfer_bytes_from_source_to_sink(
        &mut buffer,
        &mut file,
        connection,
        &mut progress_tracker,
        key,
        Mode::Encrypt,
    )
    .await?;

    progress_tracker.done();
    Ok(())
}

pub async fn transfer_tcp_to_file(
    file_path: &std::path::PathBuf,
    connection: &mut tokio::net::TcpStream,
    file_size: u64,
    key: &[u8],
) -> io::Result<()> {
    let mut file = tokio::fs::File::create(file_path).await?;
    let mut buffer = [0; BUFFER_SIZE];
    let mut progress_tracker = ProgressBarTracker::new(file_size);

    transfer_bytes_from_source_to_sink(
        &mut buffer,
        connection,
        &mut file,
        &mut progress_tracker,
        key,
        Mode::Decrypt,
    )
    .await?;

    progress_tracker.done();
    Ok(())
}
