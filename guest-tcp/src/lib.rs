wit_bindgen::generate!({
    world: "component",
});

use std::{process::exit, str::FromStr};
use wasi::{cli::environment, io::poll::Pollable, sockets::{tcp::ShutdownType, instance_network, network::{IpAddressFamily, IpSocketAddress, Ipv4SocketAddress, Network}, tcp_create_socket::{self, TcpSocket}}};
use exports::pkg::component::component_tcp::{ComponentIpSocketAddress, ComponentNetworkBorrow, ComponentPollable, ComponentTcpSocket, Guest, GuestComponentIpSocketAddress, GuestComponentNetwork, GuestComponentPollable, GuestComponentTcpSocket};

struct Component;

export!(Component);

// handle error
fn handle_error<T, E: std::fmt::Debug>(result: core::result::Result<T, E>, msg: &str) {
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

impl GuestComponentIpSocketAddress for IpSocketAddress {
    fn new(address:(u8,u8,u8,u8,),port:u16,) -> Self {
        IpSocketAddress::Ipv4(
            Ipv4SocketAddress {
                address, port,
            }
        )
    }
}

impl GuestComponentTcpSocket for TcpSocket {
    fn listen(&self,) {
        handle_error(self.start_listen(), "start_listen");
        handle_error(self.finish_listen(), "finish_listen");
    }

    fn local_address(&self,) -> ComponentIpSocketAddress {
        match self.local_address() {
            Ok(socket_address) => ComponentIpSocketAddress::new(socket_address),
            Err(e) => {
                eprintln!("local_address {:?}", e);
                exit(1)
            },
        }
    }

    fn is_listening(&self,) -> bool {
        self.is_listening()
    }

    fn subscribe(&self,) -> ComponentPollable {
        ComponentPollable::new(self.subscribe())
    }
    
    fn bind(&self, network:ComponentNetworkBorrow<'_>, socket:ComponentIpSocketAddress,) -> () {
        handle_error(self.start_bind(network.get(), *socket.get()), "start_bind");
        handle_error(self.finish_bind(), "finish_bind");
    }
}

impl GuestComponentPollable for Pollable {
    fn block(&self,) {
        self.block();
    }

    fn ready(&self,) -> bool {
        self.ready()
    }
}

impl GuestComponentNetwork for Network {
    fn new() -> Self {
        instance_network::instance_network()
    }
}

impl Guest for Component {
    type ComponentIpSocketAddress = IpSocketAddress;
    
    type ComponentTcpSocket = TcpSocket;
    
    type ComponentPollable = Pollable;
    
    type ComponentNetwork = Network;
    
    // fn get_handle_and_socket_tcp_ipv4(socket:ComponentIpSocketAddress,) -> (ComponentPollable,ComponentTcpSocket,) {
    //     todo!()
    // }
    
    // fn bind_address_to_socket_tcp_ipv4(address:(u8,u8,u8,u8,),port:u16,network:ComponentNetworkBorrow<'_>,) -> (ComponentTcpSocket,ComponentPollable,) {
    //     todo!()
    // }

    
}