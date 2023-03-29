use socket2::{Domain, SockAddr, Socket, Type};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

pub fn new(port: u32) -> UdpSocket {
    let raw_sock = Socket::new(Domain::IPV4, Type::DGRAM, None).unwrap();
    raw_sock.set_tos(0x10).unwrap();
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port as u16);
    let addr2 = SockAddr::from(addr);
    raw_sock.bind(&addr2).unwrap();
    UdpSocket::from(raw_sock)
    // UdpSocket::bind(format!("0.0.0.0:{}", port)).unwrap()
}
