use std::{process::exit, str::FromStr};
use wasi::{cli::environment, io::poll::Pollable, sockets::{tcp::ShutdownType, instance_network, network::{IpAddressFamily, IpSocketAddress, Ipv4SocketAddress, Network}, tcp_create_socket::{self, TcpSocket}}};

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

fn main() {
    let lst = environment::get_arguments();
    eprintln!("{:?}", lst);
    let (listen_address, listen_port) = resolve_address(Vec::from(lst.get(1..=5).unwrap()));
    let (connect_address, connect_port) = resolve_address(Vec::from(lst.get(6..=10).unwrap()));
    let (destination_address, destination_port) = resolve_address(Vec::from(lst.get(11..=15).unwrap()));

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
        let mut buffer: Vec<u8> = Vec::new();
        match listen_input.blocking_read(1024) {
            Ok(bytes_read) if bytes_read.len() > 0 => {
                buffer.extend_from_slice(&bytes_read);
                eprintln!("received {} bytes", bytes_read.len());
            },
            Ok(_) => {
                println!("No data received, connection might be closed.");
            },
            Err(e) => {
                eprintln!("Read failed: {:?}", e);
            },
        };

        println!("Full message received ({} bytes)", buffer.len());

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

        // todo!("process");

        // connect
        // let (connect_input, connect_output, connect_tcp_socket, connect_tcp_socket_handle) = loop {
        //     let (connect_tcp_socket, connect_tcp_socket_handle) = bind_address_to_socket_tcp_ipv4(connect_address, connect_port, &network);

        //     handle_error(connect_tcp_socket.start_connect(&network, destination_socket_address), "start_connect");
            
        //     eprintln!("connect blocking");
        //     connect_tcp_socket_handle.block();
        //     match connect_tcp_socket.finish_connect() {
        //         Ok((input, output)) => {
        //             println!("From {:?} connected to {:?}.", connect_tcp_socket.local_address(), connect_tcp_socket.remote_address());
        //             break (input, output, connect_tcp_socket, connect_tcp_socket_handle);
        //         },
        //         Err(ref e) if e.name() == "connection-refused" => {
        //             eprintln!("Failed to connect to {:?}: {} {}", destination_socket_address, e, e.message());
        //             continue;
        //         },
        //         Err(e) => {
        //             eprintln!("Failed to connect to {:?}: {} {}", destination_socket_address, e, e.message());
        //             exit(1);
        //         },
        //     };
        // };
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
        match buffer {
            // buffer if buffer == b"action/send_component".to_vec() => {
            //     if let Err(e) = connect_output.write(component_buffer.clone()) {
            //         eprintln!("Failed to send data to the other server: {}", e);
            //     } else {
            //         println!("Data sent to the other server.");
            //     }
            // },
            _ => {
                if let Err(e) = connect_output.blocking_write_and_flush(&buffer) {
                    eprintln!("Failed to send data to the other server: {}", e);
                } else {
                    println!("Data sent to the other server.");
                }
            },
        };

        handle_error(connect_tcp_socket.shutdown(ShutdownType::Both), "send socket shutdown");

        println!("Send connection closed.\n");
    }
}

// wasmtime -S inherit-network=y D:\master\thesis\aqualang\webassembly\socket\host\target\wasm32-wasip2\release\host.wasm 127 0 0 1 8080