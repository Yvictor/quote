use quote::paser::f6::bytes2f6;
// use quote::network::mcast::join_mcast;
// use std::net::{IpAddr, Ipv4Addr};

fn main() {
    // let mcast_addr: IpAddr = IpAddr::V4(Ipv4Addr::new(224, 0, 100, 100), 10000);
    // let socket = join_mcast(mcast_addr, );
    let rows: [&[u8]; 2] = [
        &[
            0x1b, 0x0, 0x41, 0x1, 0x6, 0x4, 0x0, 0x0, 0x0, 0x11, 0x30, 0x30, 0x36, 0x33, 0x32,
            0x52, 0x8, 0x30, 0x0, 0x92, 0x9, 0x15, 0x10, 0x0, 0x80, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x6, 0x32, 0x0, 0x0, 0x0, 0x0, 0x1, 0x25,
        ],
        &[
            0x1b, 0x1, 0x31, 0x1, 0x6, 0x4, 0x0, 0x10, 0x93, 0x59, 0x39, 0x31, 0x31, 0x36, 0x31,
            0x36, 0x9, 0x0, 0x0, 0x14, 0x8, 0x66, 0xda, 0x0, 0x8, 0x0, 0x0, 0x0, 0x6, 0x0, 0x0,
            0x1, 0x82, 0x0, 0x0, 0x0, 0x0, 0x6, 0x0, 0x0, 0x1, 0x82, 0x0, 0x0, 0x0, 0x0, 0x6, 0x0,
            0x0, 0x1, 0x81, 0x0, 0x0, 0x0, 0x0, 0x5, 0x0, 0x0, 0x1, 0x80, 0x0, 0x0, 0x0, 0x0, 0x16,
            0x0, 0x0, 0x1, 0x76, 0x0, 0x0, 0x0, 0x0, 0x28, 0x0, 0x0, 0x1, 0x75, 0x0, 0x0, 0x0, 0x0,
            0x20, 0x0, 0x0, 0x1, 0x93, 0x0, 0x0, 0x0, 0x0, 0x8, 0x0, 0x0, 0x1, 0x94, 0x0, 0x0, 0x0,
            0x0, 0x1, 0x0, 0x0, 0x1, 0x95, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x1, 0x96, 0x0, 0x0,
            0x0, 0x0, 0x25, 0x0, 0x0, 0x1, 0x97, 0x0, 0x0, 0x0, 0x0, 0x26, 0xc6,
        ],
    ];
    for row in rows {
        let f6 = bytes2f6(row);
        let serialized = serde_json::to_string(&f6).unwrap();
        println!("serialized = {}", serialized);
        println!("{:?}", bytes2f6(row));
    }
}
