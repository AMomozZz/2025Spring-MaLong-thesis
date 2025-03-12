use std::fmt::Display;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Serialize, Deserialize, Debug)]
pub enum PayloadType {
    Wasm,
    Component,
    Json,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PayloadQueue {
    Q1, Q2, Q3, Q4, Q5, Q6, Q7, Q8,
}

impl Display for PayloadType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PayloadType::Wasm => write!(f, "Wasm"),
            PayloadType::Component => write!(f, "Component"),
            PayloadType::Json => write!(f, "Json"),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8100").await?;
    println!("Server is listening on 127.0.0.1:8100");

    loop {
        let (mut socket, _addr) = listener.accept().await?;
        println!("New connection established! {}", _addr.to_string());
    
        tokio::spawn(async move {
            let mut temp_buf = [0u8; 1024];
            let mut payload_buffer = Vec::new();

            loop {
                match socket.read(&mut temp_buf).await {
                    Ok(0) => {
                        println!("No more data received. Closing connection.");
                        break;
                    }
                    Ok(n) => {
                        println!("Received {} bytes.", n);
                        payload_buffer.extend_from_slice(&temp_buf[..n]);
                    }
                    Err(e) => {
                        eprintln!("Failed to read from socket: {}", e);
                        break;
                    }
                }
            }
    
            println!("Full message received ({} bytes): {}", payload_buffer.len(), String::from_utf8_lossy(&payload_buffer));
    
            let response = b"Message received!";
            if let Err(e) = socket.write_all(response).await {
                eprintln!("Failed to send response: {}", e);
            } else {
                println!("Response sent to the client.");
            }
    
            println!("Connection closed.");
        });
    }
}
