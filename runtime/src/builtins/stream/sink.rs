use std::io::BufWriter;
use std::io::Write;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::path::PathBuf;

use crate::builtins::format::Format;
use crate::builtins::writer::Writer;
use crate::formats::Encode;
use crate::runner::context::Context;
use crate::traits::Data;

use super::Event;
use super::Stream;

impl<T: Data> Stream<T> {
    pub fn sink(self, ctx: &mut Context, writer: Writer, encoding: Format) {
        let mut this = self;
        let (tx, rx) = std::sync::mpsc::channel();
        ctx.sink(|| async move {
            loop {
                let event = this.recv().await;
                match event {
                    Event::Data(_, data) => tx.send(data).unwrap(),
                    Event::Watermark(_) => continue,
                    Event::Snapshot(_) => todo!(),
                    Event::Sentinel => break,
                }
            }
            Ok(())
        });
        Self::sink_encoding(ctx, rx, writer, encoding);
    }

    async fn write_pipe(
        rx: std::sync::mpsc::Receiver<T>,
        mut encoder: impl Encode + Send + 'static,
        tx: impl Write + Unpin,
    ) {
        let mut tx = BufWriter::new(tx);
        let mut buf = vec![0; 1024];
        loop {
            match rx.recv() {
                Ok(data) => match encoder.encode(&data, &mut buf) {
                    Ok(n) => {
                        tracing::info!("Encoded: {:?}", data);
                        tx.write_all(&buf[0..n]).unwrap();
                    }
                    Err(e) => tracing::info!("Failed to encode: {}", e),
                },
                Err(_) => {
                    tx.flush().unwrap();
                    break;
                }
            }
        }
    }

    async fn write_file(
        rx: std::sync::mpsc::Receiver<T>,
        path: PathBuf,
        encoder: impl Encode + Send + 'static,
    ) {
        match std::fs::File::create(&path) {
            Ok(tx) => Self::write_pipe(rx, encoder, tx).await,
            Err(e) => panic!("Failed to open file `{}`: {}", path.display(), e),
        }
    }

    async fn write_socket(
        rx: std::sync::mpsc::Receiver<T>,
        addr: SocketAddr,
        mut encoder: impl Encode + 'static,
    ) {
        println!("Connecting to {}", addr);
        let socket = TcpStream::connect(addr).expect("Failed to connect");
        println!("Connected to {}", addr);
        
        let mut writer = BufWriter::new(socket);
        let mut buf = vec![0; 1024];

        while let Ok(data) = rx.recv() {
            match encoder.encode(&data, &mut buf) {
                Ok(n) => {
                    println!("Encoded: {:?}", data);
                    let s = std::str::from_utf8(&buf[..n - 1]).unwrap();
                    println!("Sending: [{}]", s);
                    writer.write_all(s.as_bytes()).unwrap();
                    writer.write_all(b"\n").unwrap();
                    writer.flush().unwrap();
                }
                Err(e) => println!("Failed to encode: {}", e),
            }
        }
    }

    fn sink_encoding(
        ctx: &mut Context,
        rx: std::sync::mpsc::Receiver<T>,
        writer: Writer,
        encoding: Format,
    ) {
        match encoding {
            Format::Csv { sep } => {
                let encoder = crate::formats::csv::ser::Writer::new(sep);
                Self::sink_writer(ctx, rx, writer, encoder);
            }
            Format::Json => {
                let encoder = crate::formats::json::ser::Writer::new();
                Self::sink_writer(ctx, rx, writer, encoder);
            }
        }
    }

    fn sink_writer(
        ctx: &mut Context,
        rx: std::sync::mpsc::Receiver<T>,
        writer: Writer,
        encoder: impl Encode + Send + 'static,
    ) {
        ctx.spawn(async move {
            match writer {
                Writer::Stdout => Self::write_pipe(rx, encoder, std::io::stdout()).await,
                Writer::File { path } => Self::write_file(rx, path, encoder).await,
                // Writer::Http { url } => Self::write_http(rx, url, encoder).await,
                Writer::Tcp { addr } => Self::write_socket(rx, addr, encoder).await,
                Writer::Kafka { addr: _, topic: _ } => todo!(),
            }
            Ok(())
        });
    }
}
