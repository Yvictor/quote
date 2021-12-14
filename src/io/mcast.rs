use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
// use std::sync::mpsc::Sender;
use crossbeam_channel::Sender;
// use std::time::Duration;
use crate::paser::f6::{bytes2quote, bytes2header, bytes2mlen, bytes2fcode, F6};
use chrono::prelude::{Local};

fn new_socket(addr: &SocketAddr) -> io::Result<Socket> {
    let domain = if addr.is_ipv4() {
        Domain::ipv4()
    } else {
        Domain::ipv6()
    };

    let socket = Socket::new(domain, Type::dgram(), Some(Protocol::udp()))?;
    socket.set_reuse_address(true).ok();
    // socket.set
    // we're going to use read timeouts so that we don't hang waiting for packets
    // socket.set_read_timeout(Some(Duration::from_millis(100)))?;

    Ok(socket)
}

#[cfg(target_os = "linux")]
fn disable_multicast_all(udp_socket: &UdpSocket) {
    use libc::{setsockopt, IPPROTO_IP, IP_MULTICAST_ALL, c_int, socklen_t, c_void};
    use std::os::unix::io::AsRawFd;
    use std::mem;
    let raw = udp_socket.as_raw_fd();
    unsafe {
        let optval: libc::c_int = 0;
        let ret = libc::setsockopt(
            raw,
            libc::IPPROTO_IP,
            libc::IP_MULTICAST_ALL,
            &optval as *const _ as *const libc::c_void,
            mem::size_of_val(&optval) as libc::socklen_t,
        );
        if ret != 0 {
            println!("error");
        }
    }
}
#[cfg(not(target_os = "linux"))]
fn disable_multicast_all(_udp_socket: &UdpSocket) {}


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
    let udp_socket: UdpSocket = socket.into_udp_socket();

    disable_multicast_all(&udp_socket);
    Ok(udp_socket)
}

pub fn process(socket: UdpSocket, sender: &Sender<F6>){
    let mut fbuffer = [0u8; 4096];
    // let mut fbuffer_ = [0u8; 4096];
    // let mut c = Cursor::new(Vec::new());
    // let mut header = [0u8; 29];
    let filter_ip = Ipv4Addr::new(10, 3, 0, 1);
    let mut count = 0;
    let mut buf = [0u8; 512];
    let mut break_ = false;
    loop {
        match socket.recv_from(&mut fbuffer) {
            Ok((received, rec_addr)) => {
                let rec_ip = rec_addr.ip();
                match rec_ip {
                    IpAddr::V4(ref rec_ip_v4) =>{
                        if rec_ip_v4 == &filter_ip {
                            // println!("{:?}", rec_addr);
                            // println!("received {} bytes {:?}", received, &fbuffer[..received]);
                            log::debug!("received {} bytes {:?}", received, &fbuffer[..received]);
                            let mut c = Cursor::new(fbuffer);
                            c.seek(SeekFrom::Start(0)).unwrap();
                            // c.write_all(&fbuffer[..received]).unwrap();
                            let received_size = received as u64;
                            // c.write_all(&buf[..received]).unwrap();
                            loop {
                                log::debug!("{}, {}", c.position(), received_size);
                                if c.position() == received_size {
                                    break;
                                }
                                c.read_exact(&mut buf[..4]).unwrap();
                                let mlen = bytes2mlen(&buf);
                                // println!("mlen: {}", mlen);
                                c.read_exact(&mut buf[4..mlen]).unwrap();
                                log::debug!("record: {:?}", &buf[..mlen]);
                                let fcode = bytes2fcode(&buf);
                                if *fcode == 6 {
                                    log::debug!("count: {}", count);
                                    let h = bytes2header(&buf);
                                    log::debug!("header: {:?}", h);
                                    // println!("{}", count);
                                    if count == 0 {
                                        count = h.no;
                                    } else {
                                        count += 1
                                    }
                                    // println!("{} {}", count, h.no);
                                    if count != h.no {
                                        //break_ = true;
                                        //break;
                                        log::error!("count: {}, no: {}", count, h.no);
                                    }
                                    let (n_match, n_bid, n_ask) = h.n_info();
                                    let f6 = F6 {
                                        header: h,
                                        quote: bytes2quote(&buf[29..mlen], n_match, n_bid, n_ask),
                                        // received: Local::now().to_rfc3339(),
                                    };
                                    match  sender.send(f6) {
                                        Ok(_) => (),
                                        Err(e) => println!("sender error: {:?}", e)
                                    }
                                    // sender.send(f6).unwrap();
                                    // rec_handler(&f6, count);
                                }
                            }
                        }
                    }
                    IpAddr::V6(ref _rec_ip_v6) => ()
                }
                if break_ {
                    break;
                }
            }
            Err(e) => println!("recv function failed: {:?}", e),
        }
    }
}
