extern crate libc;

use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::net::{SocketAddrV6, Ipv6Addr};

pub mod ifaddrs;
use ifaddrs::interface_to_scope;

const PORT: u16 = 8513;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args();

    let _ = args.next();
    let interface = args.next().unwrap();
    let method = args.next().unwrap();

    let scope = interface_to_scope(interface.as_bytes())?
        .expect(&format!("{} not found.", interface));

    if method == "listen" {
        listener(scope)?;
    } else if method == "send" {
        sender(scope)?;
    }

    Ok(())
}

fn listener(scope: libc::c_uint) -> Result<(), Box<dyn std::error::Error>> {
    let socket = Socket::new(Domain::ipv6(), Type::dgram(), Some(Protocol::udp()))?;
    let multicast = Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0x0123);
    socket.join_multicast_v6(&multicast, scope)?;
    socket.set_only_v6(true)?;

    #[cfg(not(windows))]
    let sock_addr = SocketAddrV6::new(multicast, PORT, 0, scope);
    #[cfg(windows)]
    let sock_addr = SocketAddrV6::new(
        Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).into(), PORT, 0, scope);

    socket.bind(&SockAddr::from(sock_addr))?;

    let mut buf = [0u8; 64];
    loop {
        let (len, addr) = socket.recv_from(&mut buf)?;
        let _data = &buf[0..len];
        println!("Msg from {}", addr.as_inet6().unwrap());
    }
}
fn sender(scope: libc::c_uint) -> Result<(), Box<dyn std::error::Error>> {
    let socket = Socket::new(Domain::ipv6(), Type::dgram(), Some(Protocol::udp()))?;
    socket.set_only_v6(true)?;

    let sock_addr = SocketAddrV6::new(
        Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).into(), 0, 0, scope);
    socket.bind(&SockAddr::from(sock_addr))?;

    let multicast = Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0x0123);
    let multicast = SocketAddrV6::new(multicast, PORT, 0, scope);

    socket.send_to(b"test", &SockAddr::from(multicast))?;

    Ok(())
}
