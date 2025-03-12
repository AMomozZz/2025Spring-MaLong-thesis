// use std::{process::exit, str::FromStr};
// use wasi::cli::environment;
use wasmtime::{component::{Component, Linker, ResourceTable}, Config, Engine, Store};
use wasmtime_wasi::WasiImpl;

// const GUEST_TCP_WASI_MODULE: &[u8] = include_bytes!(concat!(
//     env!("CARGO_MANIFEST_DIR"),
//     "/../guest-tcp/target/wasm32-wasip1/release/tcp_component.wasm"
// ));

// const GUEST_RS_WASI_MODULE: &[u8] = include_bytes!(concat!(
//     env!("CARGO_MANIFEST_DIR"),
//     "/../guest-rs/target/wasm32-wasip1/release/component.wasm"
// ));

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

// // handle error
// fn handle_error<T, E: std::fmt::Debug>(result: Result<T, E>, msg: &str) {
//     match result {
//         Ok(_) => {
//             eprintln!("{} OK", msg);
//         },
//         Err(e) => {
//             eprintln!("{} {:?}", msg, e);
//             exit(1);
//         },
//     }
// }

// // socket related
// fn inner_resolve_address<T: FromStr>(option: Option<&String>) -> T {
//     match option {
//         Some(val) => match val.parse::<T>() {
//             Ok(parsed_v) => parsed_v,
//             Err(_) => {
//                 eprintln!("Failed to parse address: {:?}", val);
//                 exit(1)
//             },
//         }
//         None => {
//             eprintln!("Failed to resolve address: {:?}", option);
//             exit(1)
//         },
//     }
// }

// fn resolve_address(lst: Vec<String>) -> ((u8, u8, u8, u8), u16) {
//     // eprintln!("{:?}", lst);
//     let add1: u8 = inner_resolve_address(lst.get(0));
//     let add2: u8 = inner_resolve_address(lst.get(1));
//     let add3: u8 = inner_resolve_address(lst.get(2));
//     let add4: u8 = inner_resolve_address(lst.get(3));
//     let port: u16 = inner_resolve_address(lst.get(4));
//     ((add1, add2, add3, add4), port)
// }

// fn get_handle_and_socket_tcp_ipv4(socket_address: IpSocketAddress) -> (Pollable, TcpSocket){
    
//     let address_family = IpAddressFamily::Ipv4;
//     match tcp_create_socket::create_tcp_socket(address_family) {
//         Ok(tcp_socket) => (tcp_socket.subscribe(), tcp_socket),
//         Err(e) => {
//             eprintln!("Failed to bind to address {:?}: {}", socket_address, e);
//             exit(1)
//         },
//     }
// }

// fn bind_address_to_socket_tcp_ipv4(address: (u8, u8, u8, u8), port: u16, network: &Network) -> (TcpSocket, Pollable) {
//     let socket_address = get_socket_address(address, port);
//     let (tcp_socket_handle, tcp_socket) = get_handle_and_socket_tcp_ipv4(socket_address);
//     handle_error(tcp_socket.start_bind(network, socket_address), "start_bind");
//     handle_error(tcp_socket.finish_bind(), "finish_bind");
//     (tcp_socket, tcp_socket_handle)
// }

// main
fn main() {
    // args
    // let lst = environment::get_arguments();
    // let lst = ["host.wasm", "127", "0", "0", "1", "8080", "127", "0", "0", "1", "8090", "127", "0", "0", "1", "8100"];
    // eprintln!("{:?}", lst);
    // let (listen_address, listen_port) = resolve_address(Vec::from(lst.get(1..=5).unwrap()));
    // let (connect_address, connect_port) = resolve_address(Vec::from(lst.get(6..=10).unwrap()));
    // let (destination_address, destination_port) = resolve_address(Vec::from(lst.get(11..=15).unwrap()));
    let (listen_address, listen_port) = ((127 as u8, 0 as u8, 0 as u8, 1 as u8), 8080 as u16);
    let (connect_address, connect_port) = ((127 as u8, 0 as u8, 0 as u8, 1 as u8), 8090 as u16);
    let (destination_address, destination_port) = ((127 as u8, 0 as u8, 0 as u8, 1 as u8), 8100 as u16);

    // host runtime
    let mut config = Config::new();
    // config.async_support(true);
    let engine = Engine::new(&config).unwrap();
    let host = Host::new();

    let wi: WasiImpl<Host> = WasiImpl(wasmtime_wasi::IoImpl::<Host>(host));
    let mut store = Store::new(&engine, wi);
    let mut linker= Linker::new(&engine);
    wasmtime_wasi::add_to_linker_sync::<WasiImpl<Host>>(&mut linker).unwrap();

    // let component = Component::from_binary(&engine, &GUEST_RS_WASI_MODULE).unwrap();

    // let instance = linker.instantiate(&mut store, &component).unwrap();

    // let interface_tcp_export = instance
    //     .get_export(&mut store, None, "pkg:component/component-tcp")
    //     .unwrap();

    // eprintln!("{:?}", interface_tcp_export);

    // let resource_ip_socket_address_export = instance.get_resource(&mut store, "component-ip-socket-address").unwrap();

    // let func_ip_socket_address_constructor_export = instance.get_export(&mut store, Some(&interface_tcp_export), "")
    
    // let func_print_export = instance
    //     .get_export(&mut store, Some(&intf_export), "q1")
    //     .unwrap();
    // let func_print_typed = instance
    //     .get_typed_func::<(Vec<u8>,), (Vec<u8>,)>(&mut store, func_print_export)
    //     .unwrap();
    // let out = func_print_typed
    //     .call(&mut store, (buffer,))
    //     .unwrap().0;
    // func_print_typed.post_return(&mut store).unwrap();


}