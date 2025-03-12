use std::process::exit;

use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use wasmtime::{component::{Component, Linker, ResourceTable}, Config, Engine, Store};
use serde::{Serialize, Deserialize};

const GUEST_RS_WASI_MODULE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../guest-rs/target/wasm32-wasip2/release/component.wasm"
));

#[derive(Serialize, Deserialize, Debug)]
pub enum PayloadType {
    Wasm,
    Component,
    Json,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PayloadQueue {
    Q1 = 1, Q2, Q3, Q4, Q5, Q6, Q7, Q8,
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    msg_type: PayloadType,
    queue: PayloadQueue,
    payload: Vec<u8>,
}

// serde_json::from_slice::<PayloadType>(byte_buffer)
// serde_json::to_vec(payload)

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new();
    let engine = Engine::new(&config).unwrap();

    let component = Component::from_binary(&engine, &GUEST_RS_WASI_MODULE).unwrap();

    // let mut socket = TcpStream::connect("127.0.0.1:8100").await?;
    let mut socket = TcpStream::connect("127.0.0.1:8080").await?;

    println!("Connected to server!");
    println!("{:?}", socket.local_addr());

    // let payload = r#"{"Bid":{"auction":1000,"bidder":1001,"price":2419091,"channel":"Google","url":"https://www.nexmark.com/vzbh/ewxa/yha/item.htm?query=1","date_time":1740736239576,"extra":"rthoyqrqsnaalanrzuvulspeumufvvwpfwczanrzowgwrphioovqxuvkzlqq"}}"#.as_bytes().to_vec();
    // let message = Message {
    //     msg_type: PayloadType::Json,
    //     queue: PayloadQueue::Q1,
    //     payload: payload.clone(),
    // };

    let payload = match component.serialize() {
        Ok(component_buffer) => {
            component_buffer
        },
        Err(e) => {
            panic!("Failed to serialize component: {}", e)
        },
    };
    let message = Message {
        msg_type: PayloadType::Component,
        queue: PayloadQueue::Q1,
        payload: payload.clone(),
    };

    let serialized_message = serde_json::to_vec(&message).unwrap();
    socket.write_all(&serialized_message).await?;
    socket.flush().await?;

    println!("Sent message: {:?}", (message.msg_type, message.queue, message.payload.len()));
    socket.shutdown().await?;

    let mut buffer = [0u8; 1024];
    let n = socket.read(&mut buffer).await?;
    if n > 0 {
        println!("Received from server: {:?}", &buffer[..n]);
    }
    
    Ok(())
}