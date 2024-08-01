use aes_gcm::{aead::Aead, Aes256Gcm, Key, KeyInit, Nonce};
use rand::{rngs::OsRng, RngCore};
use std::io;
use std::pin::Pin;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::ui::ProgressBarTracker;

// constant buffer size
const BUFFER_SIZE: usize = 1024;

pub trait UpdateProgress {
    fn update_progress(&mut self, bytes_read: u64);
}

pub enum Mode {
    Encrypt,
    Decrypt,
}

async fn transfer_bytes_from_source_to_sink(
    mut buffer: &mut [u8],
    source: &mut (dyn tokio::io::AsyncRead + Unpin),
    sink: &mut (dyn tokio::io::AsyncWrite + Unpin),
    progress_tracker: &mut dyn UpdateProgress,
    key: &[u8],
    mode: Mode,
) -> io::Result<()> {
    let mut bytes_read = 0;

    let encryption_key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(encryption_key);

    let mut pinned_source = Pin::new(source);
    let mut pinned_sink = Pin::new(sink);

    match mode {
        Mode::Encrypt => {
            println!("Encrypting");
            let mut nonce_bytes = [0u8; 12];

            while let Ok(n) = pinned_source.as_mut().read(&mut buffer).await {
                if n == 0 {
                    break;
                }

                OsRng.fill_bytes(&mut nonce_bytes);
                let nonce = Nonce::from_slice(&nonce_bytes);

                // Encrypt the buffer
                let encrypted_buffer =
                    cipher.encrypt(nonce, &buffer[..n]).unwrap();

                // Get the length of encrypted buffer and fit into 4 bytes
                let encrypted_buffer_len =
                    (encrypted_buffer.len() as u32).to_be_bytes();

                let mut combined_buffer =
                    Vec::with_capacity(4 + 12 + encrypted_buffer.len());
                combined_buffer.extend_from_slice(&encrypted_buffer_len);
                combined_buffer.extend_from_slice(&nonce);
                combined_buffer.extend_from_slice(&encrypted_buffer);

                pinned_sink.write_all(&mut combined_buffer).await?;

                bytes_read += n;
                progress_tracker.update_progress(bytes_read as u64);
            }
        }
        Mode::Decrypt => {
            println!("Decrypting");
            let mut temp_buffer = vec![];

            while let Ok(n) = pinned_source.as_mut().read(&mut buffer).await {
                if n == 0 {
                    break;
                }

                temp_buffer.extend_from_slice(&buffer[..n]);

                while temp_buffer.len() >= 16 {
                    if temp_buffer.len() < 16 {
                        break;
                    }

                    // Get the data length hby slicing first 4 bytes
                    let data_length: usize = temp_buffer[0..4]
                        .try_into()
                        .map(u32::from_be_bytes)
                        .unwrap()
                        .try_into()
                        .unwrap();
                    let total_length = 4 + 12 + data_length;

                    if temp_buffer.len() < total_length {
                        break;
                    }

                    let nonce_bytes = &temp_buffer[4..16];
                    let encrypted_data = &temp_buffer[16..total_length];

                    let nonce = Nonce::from_slice(nonce_bytes);
                    let decrypted_data =
                        cipher.decrypt(&nonce, encrypted_data).unwrap();

                    pinned_sink.write_all(&decrypted_data).await?;
                    bytes_read += decrypted_data.len();
                    progress_tracker.update_progress(bytes_read as u64);

                    temp_buffer.drain(..total_length);
                }
            }
        }
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
