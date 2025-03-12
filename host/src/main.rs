use std::{collections::LinkedList, env::args, fmt::Display, net::{IpAddr, Ipv4Addr, Shutdown, SocketAddr}, process::exit, str::FromStr};
use serde::{Deserialize, Serialize};
// use wasi::{cli::environment, io::poll::Pollable, sockets::{tcp::ShutdownType, instance_network, network::{IpAddressFamily, IpSocketAddress, Ipv4SocketAddress, Network}, tcp_create_socket::{self, TcpSocket}}};
use cap_net_ext::AddressFamily;
use tokio_util::bytes::Bytes;
use wasmtime::{component::{Component, Linker, ResourceTable}, Config, Engine, Store};
use wasmtime_wasi::{Pollable, bindings::sockets::tcp_create_socket::TcpSocket, WasiImpl};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadType {
    Wasm,
    Component,
    Json,
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadQueue {
    Q1 = 1, Q2, Q3, Q4, Q5, Q6, Q7, Q8,
}

impl Display for PayloadQueue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Queue {}", *self as usize)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    msg_type: PayloadType,
    queue: PayloadQueue,
    payload: Vec<u8>,
}

// host
struct Host {
    ctx: wasmtime_wasi::WasiCtx,
    table: ResourceTable,
}

impl wasmtime_wasi::WasiView for Host {
    fn ctx(&mut self) -> &mut wasmtime_wasi::WasiCtx {
        &mut self.ctx
    }
}

impl wasmtime_wasi::IoView for Host {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

impl Host {
    fn new() -> Self {
        let ctx = wasmtime_wasi::WasiCtxBuilder::new().inherit_stdio().build();
        let table = ResourceTable::new();
        Self { ctx, table }
    }
}

// handle error
fn handle_error<T, E: std::fmt::Debug>(result: Result<T, E>, msg: &str) {
    match result {
        Ok(_) => {
            eprintln!("{} OK", msg);
        },
        Err(e) => {
            eprintln!("{} {:?}", msg, e);
            exit(1);
        },
    }
}

// socket related
fn inner_resolve_address<T: FromStr>(option: Option<&String>) -> T {
    match option {
        Some(val) => match val.parse::<T>() {
            Ok(parsed_v) => parsed_v,
            Err(_) => {
                eprintln!("Failed to parse address: {:?}", val);
                exit(1)
            },
        }
        None => {
            eprintln!("Failed to resolve address: {:?}", option);
            exit(1)
        },
    }
}

fn resolve_address(lst: Vec<String>) -> ((u8, u8, u8, u8), u16) {
    // eprintln!("{:?}", lst);
    let add1: u8 = inner_resolve_address(lst.get(0));
    let add2: u8 = inner_resolve_address(lst.get(1));
    let add3: u8 = inner_resolve_address(lst.get(2));
    let add4: u8 = inner_resolve_address(lst.get(3));
    let port: u16 = inner_resolve_address(lst.get(4));
    ((add1, add2, add3, add4), port)
}

fn get_socket_address(address: (u8, u8, u8, u8), port: u16) -> (AddressFamily, SocketAddr) {
    let ip_address = IpAddr::V4(Ipv4Addr::new(address.0, address.1, address.2, address.3));
    (AddressFamily::of_ip_addr(ip_address), SocketAddr::new(ip_address, port))
}

fn bind_address_to_socket_tcp_ipv4(address: (u8, u8, u8, u8), port: u16) -> TcpSocket {
    let (family, socket_address) = get_socket_address(address, port);
    let mut socket = TcpSocket::new(family).unwrap();
    handle_error(socket.start_bind(socket_address), "start_bind");
    handle_error(socket.finish_bind(), "finish_bind");
    socket
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lst: Vec<String> = args().collect();
    eprintln!("{:?}", lst);
    let (listen_address, listen_port, connect_address, connect_port, destination_address, destination_port) = match lst.len() {
        n if n > 1 => {
            let (listen_address, listen_port) = resolve_address(Vec::from(lst.get(1..=5).unwrap()));
            let (connect_address, connect_port) = resolve_address(Vec::from(lst.get(6..=10).unwrap()));
            let (destination_address, destination_port) = resolve_address(Vec::from(lst.get(11..=15).unwrap()));
            (listen_address, listen_port, connect_address, connect_port, destination_address, destination_port)
        },
        _ => ((127, 0, 0, 1), 8080, (127, 0, 0, 1), 8090, (127, 0, 0, 1), 8100),
    };

    let mut config = Config::new();
    // config.async_support(true);
    let engine = Engine::new(&config).unwrap();
    let host = Host::new();

    let wi: WasiImpl<Host> = WasiImpl(wasmtime_wasi::IoImpl::<Host>(host));
    let mut store = Store::new(&engine, wi);
    let mut linker= Linker::new(&engine);
    wasmtime_wasi::add_to_linker_sync::<WasiImpl<Host>>(&mut linker).unwrap();

    let mut listen_tcp_socket = bind_address_to_socket_tcp_ipv4(listen_address, listen_port);
    let (destination_address_family, destination_socket_address) = get_socket_address(destination_address, destination_port);

    handle_error(listen_tcp_socket.start_listen(), "start_listen");
    handle_error(listen_tcp_socket.finish_listen(), "finish_listen");

    let listening_address = match listen_tcp_socket.local_address() {
        Ok(socket_address) => socket_address,
        Err(e) => {
            eprintln!("local_address {:?}", e);
            exit(1)
        },
    };

    match listen_tcp_socket.is_listening() {
        true => eprintln!("Server listening on {:?}", listening_address),
        false => eprintln!("Server not on listening state")
    }

    let mut queue_type: Option<PayloadQueue> = None;
    let mut component_container: Option<Component> = None;
    let mut data_container: LinkedList<Message> = LinkedList::new();
    let mut out_container: LinkedList<Vec<u8>> = LinkedList::new();

    loop {
        // accept
        eprintln!("\nlisten blocking");
        listen_tcp_socket.ready().await;
        let (listen_tcp_socket, mut listen_input, mut listen_output) = match listen_tcp_socket.accept() {
            Ok(tuple) => tuple,
            Err(e) => {
                eprintln!("Accept failed: {:?}\n", e);
                exit(1);
            },
        };
        let local_address = match listen_tcp_socket.local_address() {
            Ok(addr) => addr,
            Err(e) => {
                eprintln!("local_address {:?}", e);
                exit(1);
            },
        };
        let remote_address = match listen_tcp_socket.remote_address() {
            Ok(addr) => addr,
            Err(e) => {
                eprintln!("remote_address {:?}", e);
                exit(1);
            },
        };
        eprintln!("new connection from {:?} to {:?}", remote_address, local_address);
        
        // read
        let mut buffer = Vec::new();

        loop {
            match listen_input.blocking_read(1024).await {
                Ok(bytes_read) if !bytes_read.is_empty() => {
                    buffer.extend_from_slice(&bytes_read);
                    // eprintln!("received {} bytes", bytes_read.len());
                },
                Ok(_) => {
                    println!("No data received, connection might be closed.");      // TODO: need some change of the closed connection
                    break;
                },
                Err(e) => {
                    eprintln!("Read payload failed: {:?}", e);
                    break;
                },
            };
        }

        println!("Full message received ({} bytes)", buffer.len()); // , serde_json::from_slice::<Message>(&buffer).unwrap()

        let response = Bytes::from("Message received!");
        if let Err(e) = listen_output.blocking_write_and_flush(response).await {
            eprintln!("Failed to send response: {}", e);
        } else {
            println!("Response sent to the client.");
        }

        handle_error(listen_tcp_socket.shutdown(Shutdown::Both), "listen socket shutdown");
        println!("Receive connection closed.\n");

        // parse and process
        println!("starting process");
        match serde_json::from_slice::<Message>(&buffer) {
            Ok(message) if message.msg_type == PayloadType::Component => {
                component_container = Some(unsafe {
                    Component::deserialize(&engine, message.payload).unwrap()
                });
                queue_type = Some(message.queue);
                loop {
                    match data_container.pop_front() {
                        Some(data_message) => {
                            let component = component_container.as_ref().unwrap();
                            let instance = linker.instantiate(&mut store, component).unwrap();
                            let intf_export = instance
                                .get_export(&mut store, None, "pkg:component/nexmark")
                                .unwrap();
                            // eprintln!("{:?}", intf_export);
                            if message.queue != data_message.queue {
                                eprintln!("queue not match (data:{}, component: {}), skip", data_message.queue, queue_type.unwrap());
                                continue;
                            }
                            let func_print_export = instance
                                .get_export(&mut store, Some(&intf_export), format!("q{}", data_message.queue as u8).as_str())
                                .unwrap();
                            let func_print_typed = instance
                                .get_typed_func::<(Vec<u8>,), (Vec<u8>,)>(&mut store, func_print_export)
                                .unwrap();
                            // eprintln!("{:?}", data_container.front());
                            let out = func_print_typed
                                .call(&mut store, (data_message.payload,))
                                .unwrap().0;
                            func_print_typed.post_return(&mut store).unwrap();
                            
                            out_container.push_back(out);
                            eprintln!("process finish: {:?}\n", out_container.back());
                        },
                        None => {
                            eprintln!("Component for {} stored in container", message.queue.to_string());
                            break;
                        },
                    }
                }
                continue;
            },
            Ok(message) if message.msg_type == PayloadType::Wasm => {
                todo!();
            },
            Ok(message) => {
                match component_container {
                    Some(ref component) => {
                        let instance = linker.instantiate(&mut store, component).unwrap();
                        let intf_export = instance
                            .get_export(&mut store, None, "pkg:component/nexmark")
                            .unwrap();
                        // eprintln!("{:?}", intf_export);
                        let func_print_export = instance
                            .get_export(&mut store, Some(&intf_export), format!("q{}", message.queue as u8).as_str())
                            .unwrap();
                        let func_print_typed = instance
                            .get_typed_func::<(Vec<u8>,), (Vec<u8>,)>(&mut store, func_print_export)
                            .unwrap();
                        // eprintln!("{:?}", data_container.front());
                        let out = func_print_typed
                            .call(&mut store, (message.payload.clone(),))
                            .unwrap().0;
                        func_print_typed.post_return(&mut store).unwrap();
                        
                        out_container.push_back(out);
                        eprintln!("process finish: {:?}\n", out_container.back());
                    },
                    None => {
                        data_container.push_back(message);
                        println!("{} messages waiting component to start process.", data_container.len());
                    },
                };
            },
            Err(e) => {
                eprintln!("Failed to deserialize component: {}", e);
            },
        }

        // forward
        if out_container.is_empty() {
            continue;
        }
        else {
            // connect
            let mut connect_tcp_socket = bind_address_to_socket_tcp_ipv4(connect_address, connect_port);

            handle_error(connect_tcp_socket.start_connect(destination_socket_address), "start_connect");
            
            eprintln!("connect blocking");
            connect_tcp_socket.ready().await;
            let (connect_input, mut connect_output) = match connect_tcp_socket.finish_connect() {
                Ok((input, output)) => {
                    println!("From {:?} connected to {:?}.", connect_tcp_socket.local_address(), connect_tcp_socket.remote_address());
                    (input, output)
                },
                Err(e) => {
                    eprintln!("Failed to connect to {:?}: {}", destination_socket_address, e);
                    exit(1);
                },
            };

            // send
            loop {
                match out_container.pop_front() {
                    Some(out ) => {
                        if let Err(e) = connect_output.blocking_write_and_flush(Bytes::from(out)).await {
                            eprintln!("Failed to send data to the other server: {}", e);
                        } else {
                            println!("Data sent to the other server.");
                        }
                    },
                    None => {
                        println!("No more data to send.");
                        break;
                    },
                };
            }

            // close
            handle_error(connect_tcp_socket.shutdown(Shutdown::Both), "send socket shutdown");
            println!("Send connection closed.\n");
        }
    }
    // exit(0);
}

        // match listen_input.blocking_read(1024).await {
        //     Ok(bytes_read) if !bytes_read.is_empty() => {
        //         type_buffer.extend_from_slice(&bytes_read);
        //         eprintln!("received {} bytes: {:?}", bytes_read.len(), String::from_utf8(bytes_read.to_vec()));
        //     },
        //     Ok(_) => {
        //         println!("No data received, connection might be closed.");
        //         handle_error(listen_tcp_socket.shutdown(Shutdown::Both), "listen socket shutdown");
        //         println!("Receive connection closed.\n");
        //     },
        //     Err(e) => {
        //         eprintln!("Read type failed: {:?}", e);
        //         exit(1);
        //     },
        // };

        // match listen_input.blocking_read(1024).await {
        //     Ok(bytes_read) if !bytes_read.is_empty() => {
        //         queue_buffer.extend_from_slice(&bytes_read);
        //         eprintln!("received {} bytes: {:?}", bytes_read.len(), String::from_utf8(bytes_read.to_vec()));
        //     },
        //     Ok(_) => {
        //         println!("No data received, connection might be closed.");
        //         handle_error(listen_tcp_socket.shutdown(Shutdown::Both), "listen socket shutdown");
        //         println!("Receive connection closed.\n");
        //     },
        //     Err(e) => {
        //         eprintln!("Read queue failed: {:?}", e);
        //         exit(1);
        //     },
        // };

        // let instance = match component_container {
        //     Some(ref component) => linker.instantiate(&mut store, component).unwrap(),
        //     None => panic!("component_container is None!"),
        // };