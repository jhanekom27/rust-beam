use aes_gcm::{aead::Aead, Aes256Gcm, Key, KeyInit, Nonce};
use rand::{rngs::OsRng, RngCore};
use std::io;
use std::pin::Pin;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

use crate::ui::ProgressBarTracker;

// BIGGEST PERFORMANCE WIN: Increased buffer size from 1KB to 64KB
// This reduces system calls, network packets, and encryption operations by 64x
const BUFFER_SIZE: usize = 64 * 1024;

pub trait UpdateProgress {
    fn update_progress(&mut self, bytes_read: u64);
}

pub enum Mode {
    Encrypt,
    Decrypt,
}

/// Optimize TCP socket for high-throughput file transfers
pub fn optimize_tcp_socket(stream: &TcpStream) -> io::Result<()> {
    // Disable Nagle's algorithm for lower latency
    // This sends packets immediately instead of buffering small writes
    stream.set_nodelay(true)?;
    
    // Note: For socket buffer sizes, they're typically auto-tuned by the OS
    // but you can manually set them if needed with low-level socket options
    
    Ok(())
}

/// Advanced TCP optimizations for maximum network throughput
pub fn optimize_tcp_socket_advanced(stream: &TcpStream) -> io::Result<()> {
    // Basic optimization
    stream.set_nodelay(true)?;
    
    // For very high-throughput scenarios, you might want to tune socket buffers
    // Note: These require platform-specific code and are usually auto-tuned by modern OS
    
    Ok(())
}

/// Batched write optimization - accumulate small writes before sending
pub struct BatchedWriter {
    stream: TcpStream,
    buffer: Vec<u8>,
    batch_size: usize,
}

impl BatchedWriter {
    pub fn new(stream: TcpStream, batch_size: usize) -> Self {
        Self {
            stream,
            buffer: Vec::with_capacity(batch_size),
            batch_size,
        }
    }
    
    pub async fn write(&mut self, data: &[u8]) -> io::Result<()> {
        self.buffer.extend_from_slice(data);
        
        if self.buffer.len() >= self.batch_size {
            self.flush().await?;
        }
        
        Ok(())
    }
    
    pub async fn flush(&mut self) -> io::Result<()> {
        if !self.buffer.is_empty() {
            self.stream.write_all(&self.buffer).await?;
            self.buffer.clear();
        }
        Ok(())
    }
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
                let encrypted_buffer = cipher.encrypt(nonce, &buffer[..n]).unwrap();

                // Get the length of encrypted buffer and fit into 4 bytes
                let encrypted_buffer_len = (encrypted_buffer.len() as u32).to_be_bytes();

                let mut combined_buffer = Vec::with_capacity(4 + 12 + encrypted_buffer.len());
                combined_buffer.extend_from_slice(&encrypted_buffer_len);
                combined_buffer.extend_from_slice(&nonce_bytes);
                combined_buffer.extend_from_slice(&encrypted_buffer);

                pinned_sink.write_all(&combined_buffer).await?;

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

                    // Get the data length by slicing first 4 bytes
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
                    let decrypted_data = cipher.decrypt(&nonce, encrypted_data).unwrap();

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
    // Optimize TCP socket for better network performance
    optimize_tcp_socket(connection)?;
    
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
    // Optimize TCP socket for better network performance  
    optimize_tcp_socket(connection)?;
    
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

/// Optimized file transfer with async pipelining for network-bound scenarios
/// Overlaps read/encrypt/write operations to keep the network pipe full
pub async fn transfer_file_to_tcp_optimized(
    file_path: &std::path::PathBuf,
    connection: &mut tokio::net::TcpStream,
    key: &[u8],
) -> io::Result<()> {
    // Optimize TCP socket first
    optimize_tcp_socket(connection)?;
    
    let file = tokio::fs::File::open(file_path).await?;
    let file_size = file.metadata().await?.len();
    let mut progress_tracker = ProgressBarTracker::new(file_size);

    // Channel for pipelining - larger buffer for network-bound scenarios
    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(8); // Increased buffer for network
    
    let encryption_key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(&encryption_key);
    
    // Spawn encryption task
    let encrypt_handle = tokio::spawn(async move {
        let mut file = file;
        let mut buffer = vec![0u8; BUFFER_SIZE];
        let mut nonce_bytes = [0u8; 12];
        
        while let Ok(n) = file.read(&mut buffer).await {
            if n == 0 {
                break;
            }
            
            OsRng.fill_bytes(&mut nonce_bytes);
            let nonce = Nonce::from_slice(&nonce_bytes);
            
            // Encrypt the chunk
            let encrypted_data = cipher.encrypt(nonce, &buffer[..n]).unwrap();
            
            // Build the packet format
            let encrypted_len = (encrypted_data.len() as u32).to_be_bytes();
            let mut packet = Vec::with_capacity(4 + 12 + encrypted_data.len());
            packet.extend_from_slice(&encrypted_len);
            packet.extend_from_slice(&nonce_bytes);
            packet.extend_from_slice(&encrypted_data);
            
            if tx.send(packet).await.is_err() {
                break; // Receiver dropped
            }
        }
    });
    
    // Write task (main thread) - overlaps with encryption
    let mut total_written = 0u64;
    while let Some(packet) = rx.recv().await {
        connection.write_all(&packet).await?;
        total_written += packet.len() as u64;
        progress_tracker.update_progress(total_written);
    }
    
    // Wait for encryption to complete
    let _ = encrypt_handle.await;
    
    progress_tracker.done();
    Ok(())
}
