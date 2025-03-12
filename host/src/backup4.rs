use std::{fmt::Display, process::exit, str::FromStr};
use serde::{Deserialize, Serialize};
// use wasi::{cli::environment, io::poll::Pollable, sockets::{tcp::ShutdownType, instance_network, network::{IpAddressFamily, IpSocketAddress, Ipv4SocketAddress, Network}, tcp_create_socket::{self, TcpSocket}}};
use wasmtime::{component::{Component, Linker, ResourceTable}, Config, Engine, Store};
use wasmtime_wasi::WasiImpl;

#[derive(Serialize, Deserialize, Debug)]
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

fn get_socket_address(address: (u8, u8, u8, u8), port: u16) -> IpSocketAddress {
    IpSocketAddress::Ipv4(
        Ipv4SocketAddress {
            address, port,
        }
    )
}

fn get_handle_and_socket_tcp_ipv4(socket_address: IpSocketAddress) -> (Pollable, TcpSocket){
    
    let address_family = IpAddressFamily::Ipv4;
    match tcp_create_socket::create_tcp_socket(address_family) {
        Ok(tcp_socket) => (tcp_socket.subscribe(), tcp_socket),
        Err(e) => {
            eprintln!("Failed to bind to address {:?}: {}", socket_address, e);
            exit(1)
        },
    }
}

fn bind_address_to_socket_tcp_ipv4(address: (u8, u8, u8, u8), port: u16, network: &Network) -> (TcpSocket, Pollable) {
    let socket_address = get_socket_address(address, port);
    let (tcp_socket_handle, tcp_socket) = get_handle_and_socket_tcp_ipv4(socket_address);
    handle_error(tcp_socket.start_bind(network, socket_address), "start_bind");
    handle_error(tcp_socket.finish_bind(), "finish_bind");
    (tcp_socket, tcp_socket_handle)
}

// main
fn main() {
    // args
    // let lst = environment::get_arguments();
    // eprintln!("{:?}", lst);
    // let (listen_address, listen_port) = resolve_address(Vec::from(lst.get(1..=5).unwrap()));
    // let (connect_address, connect_port) = resolve_address(Vec::from(lst.get(6..=10).unwrap()));
    // let (destination_address, destination_port) = resolve_address(Vec::from(lst.get(11..=15).unwrap()));

    let (listen_address, listen_port) = ((127, 0, 0, 1), 8080);
    let (connect_address, connect_port) = ((127, 0, 0, 1), 8090);
    let (destination_address, destination_port) = ((127, 0, 0, 1), 8100);

    // host runtime
    let mut config = Config::new();
    // config.async_support(true);
    let engine = Engine::new(&config).unwrap();
    let host = Host::new();

    let wi: WasiImpl<Host> = WasiImpl(wasmtime_wasi::IoImpl::<Host>(host));
    let mut store = Store::new(&engine, wi);
    let mut linker= Linker::new(&engine);
    wasmtime_wasi::add_to_linker_sync::<WasiImpl<Host>>(&mut linker).unwrap();

    // socket
    let network = instance_network::instance_network();

    let (listen_tcp_socket, listen_tcp_socket_handle) = bind_address_to_socket_tcp_ipv4(listen_address, listen_port, &network);
    let destination_socket_address = get_socket_address(destination_address, destination_port);

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
        true => eprintln!("Server listening on {:?}\n", listening_address),
        false => eprintln!("Server not on listening state")
    }

    let mut component_container: Option<Component> = None;
    let mut data_container: Option<Vec<u8>> = None;

    loop {
        // accept
        eprintln!("listen blocking");
        listen_tcp_socket_handle.block();
        let (listen_tcp_socket, listen_input, listen_output) = match listen_tcp_socket.accept() {
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
        let mut type_buffer = Vec::new();
        let mut payload_buffer = Vec::new();

        match listen_input.blocking_read(1024) {
            Ok(bytes_read) if bytes_read.len() > 0 => {
                type_buffer.extend_from_slice(&bytes_read);
                eprintln!("received {} bytes", bytes_read.len());
                
                match serde_json::from_slice::<PayloadType>(&type_buffer) {
                    Ok(PayloadType::Component) => {
                        todo!();
                    },
                    Ok(PayloadType::Wasm) => {
                        todo!();
                    },
                    Ok(PayloadType::Json) => {
                        match component_container {
                            Some(ref component) => {
                                todo!();
                            },
                            None => {
                                data_container = Some(payload_buffer.clone());
                                println!("Waiting component to start process.");
                            },
                        };
                    },
                    Err(e) => {
                        eprintln!("Failed to deserialize component: {}", e);
                    },
                }
            },
            Ok(_) => {
                println!("No data received, connection might be closed.");
            },
            Err(e) => {
                eprintln!("Read failed: {:?}", e);
            },
        };

        loop {
            match listen_input.blocking_read(1024) {
                Ok(bytes_read) if bytes_read.len() > 0 => {
                    payload_buffer.extend_from_slice(&bytes_read);
                    eprintln!("received {} bytes", bytes_read.len());
                    break;
                },
                Ok(_) => {
                    println!("No data received, connection might be closed.");
                },
                Err(e) => {
                    eprintln!("Read failed: {:?}", e);
                    exit(1);
                },
            };
        }

        println!("Full message received ({} bytes)", payload_buffer.len());

        let response = b"Message received!";
        if let Err(e) = listen_output.blocking_write_and_flush(response) {
            eprintln!("Failed to send response: {}", e);
        } else {
            println!("Response sent to the client.");
        }
        handle_error(listen_output.flush(), "output flush");

        // close
        handle_error(listen_tcp_socket.shutdown(ShutdownType::Both), "listen socket shutdown");
        println!("Receive connection closed.\n");

        // proces
        println!("starting process");
        todo!();
        component_container = Some(unsafe {
            Component::deserialize(&engine, &payload_buffer).unwrap()
        });

        let instance = linker.instantiate(&mut store, component).unwrap();

        let intf_export = instance
            .get_export(&mut store, None, "pkg:component/nexmark")
            .unwrap();

        let func_print_export = instance
            .get_export(&mut store, Some(&intf_export), "q1")
            .unwrap();
        let func_print_typed = instance
            .get_typed_func::<(Vec<u8>,), (Vec<u8>,)>(&mut store, func_print_export)
            .unwrap();
        let out = func_print_typed
            .call(&mut store, (data_container.clone().expect("Unexpected None value try to pass to .wasm"),))
            .unwrap().0;
        func_print_typed.post_return(&mut store).unwrap();

        eprintln!("process finish: {:?}\n", out);

        // connect
        let (connect_tcp_socket, connect_tcp_socket_handle) = bind_address_to_socket_tcp_ipv4(connect_address, connect_port, &network);

        handle_error(connect_tcp_socket.start_connect(&network, destination_socket_address), "start_connect");
        
        eprintln!("connect blocking");
        connect_tcp_socket_handle.block();
        let (connect_input, connect_output) = match connect_tcp_socket.finish_connect() {
            Ok((input, output)) => {
                println!("From {:?} connected to {:?}.", connect_tcp_socket.local_address(), connect_tcp_socket.remote_address());
                (input, output)
            },
            Err(e) => {
                eprintln!("Failed to connect to {:?}: {} {}", destination_socket_address, e, e.message());
                exit(1);
            },
        };

        // send
        match out {
            _ => {
                if let Err(e) = connect_output.blocking_write_and_flush(&out) {
                    eprintln!("Failed to send data to the other server: {}", e);
                } else {
                    println!("Data sent to the other server.");
                }
            },
        };

        // close
        handle_error(connect_tcp_socket.shutdown(ShutdownType::Both), "send socket shutdown");
        println!("Send connection closed.\n");
    }
}