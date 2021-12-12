use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
// use std::time::Duration;
use crate::paser::f6::{bytes2quote, bytes2header, F6};

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
        IpAddr::V4(ref mdns_v4) => match ip_interface {
            IpAddr::V4(ref if_v4) => {
                socket.join_multicast_v4(mdns_v4, if_v4).unwrap();
            }
            IpAddr::V6(ref _if_v6) => (),
        },
        IpAddr::V6(ref mdns_v6) => {
            socket.join_multicast_v6(mdns_v6, 0).unwrap();
        }
    }
    socket.bind(&SockAddr::from(addr)).unwrap();

    Ok(socket.into_udp_socket())
}

pub fn process(socket: UdpSocket, rec_handler: fn(F6)){
    let mut buf = [0u8; 4096];
    let mut c = Cursor::new(Vec::new());
    let mut header = [0u8; 29];
    let filter_ip = Ipv4Addr::new(10, 3, 0, 1);
    loop {
        match socket.recv_from(&mut buf) {
            Ok((received, rec_addr)) => {
                let rec_ip = rec_addr.ip();
                match rec_ip {
                    IpAddr::V4(ref rec_ip_v4) =>{
                        if rec_ip_v4 == &filter_ip {
                            println!("received {} bytes {:?}", received, &buf[..received]);
                            c.seek(SeekFrom::Start(0)).unwrap();
                            c.write_all(&buf[..received]).unwrap();
                            let vec_len = c.position();
                            c.seek(SeekFrom::Start(0)).unwrap();
                            while c.position() < vec_len {
                                c.read_exact(&mut header).unwrap();
                                // let h = bytes2header(&buf[..29]);
                                let h = bytes2header(&header);
                                println!("header: {:?}", h);
                                let (n_match, n_bid, n_ask) = h.n_info();     
                                let mut body = vec![0u8; (h.mlen - 29) as usize];
                                c.read_exact(&mut body).unwrap(); 
                                println!("body: {:?}", body);
                                let f6 = F6 {
                                    header: h,
                                    quote: bytes2quote(&body, n_match, n_bid, n_ask),
                                };
                                rec_handler(f6);
                                // println!("{:?}", f6);
                            } 

                        }
                    }
                    IpAddr::V6(ref _rec_ip_v6) => ()
                }
            }
            Err(e) => println!("recv function failed: {:?}", e),
        }
    }
}
