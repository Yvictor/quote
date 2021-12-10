use std::io;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
// use std::time::Duration;

fn new_socket(addr: &SocketAddr) -> io::Result<Socket> {
    let domain = if addr.is_ipv4() {
        Domain::ipv4()
    } else {
        Domain::ipv6()
    };

    let socket = Socket::new(domain, Type::dgram(), Some(Protocol::udp()))?;
    socket.set_reuse_address(true).ok();
    // we're going to use read timeouts so that we don't hang waiting for packets
    // socket.set_read_timeout(Some(Duration::from_millis(100)))?;

    Ok(socket)
}
pub fn join_mcast(addr: SocketAddr, interface: SocketAddr) -> io::Result<UdpSocket> {
    let ip_arrd = addr.ip();
    let ip_interface = interface.ip();
    let socket = new_socket(&addr).unwrap();
    match ip_arrd {
        IpAddr::V4(ref mdns_v4) => {
            socket.join_multicast_v4(mdns_v4, &ip_interface).unwrap();
        }
        IpAddr::V6(ref mdns_v6) => {
            socket.join_multicast_v6(mdns_v6, &ip_interface).unwrap();
        }
    }
    socket.bind(&SockAddr::from(addr));

    Ok(socket.into_udp_socket())
}