// use quote::paser::f6::bytes2f6;
use quote::io::fs::{readf6file, readf6filebuffer};
use quote::paser::f6::{F6, F6Received};
use quote::io::mcast::{join_mcast, process};
use quote::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;
use std::io::Write;
use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
// use log::info;
// use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use crossbeam_channel::{bounded, Sender, Receiver};
extern crate redis;

fn main() {
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S.%f"),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Error)
        .init();
    fn f6handler(f6: F6, count: u64) {
        log::info!("{:?}", f6);
        // count += 1;
        // let diff = f6.header.no - count;
        // if diff != 0{
        //     println!("loss count: {}", diff);
        // }
    }
    // let f6handler = Box::new(|f6: F6| {
    //     count += 1;
    //     println!("{:?}", f6.header.no - count);
    // });

    // let path = Path::new("tests/data/f6_01000001_01001000_TP03.new");
    // // let path = Path::new("集中市場行情格式六_04000001_04500000_TP09.new");
    // let display = path.display();
    // log::info!("start parsing file: {}", display);
    // // readf6file(&Path::new("集中市場行情格式六_01000001_01500000_TP03.new"), f6handler);
    // readf6file(&path, f6handler);
    // log::info!("readf6file");
    // readf6filebuffer(&path, f6handler);
    // log::info!("finish");
    
    // let (sender, receiver): (Sender<F6>, Receiver<F6>) = channel();
    let (sender, receiver): (Sender<F6>, Receiver<F6>) = bounded(4096);

    let mcast_addr: SocketAddr = SocketAddr::new(IpAddr::from(Ipv4Addr::new(224, 0, 100, 100)), 10000);
    let if_addr: SocketAddr = SocketAddr::new(IpAddr::from(Ipv4Addr::new(192, 168, 32, 23)), 10000);
    let socket = join_mcast(mcast_addr, if_addr).unwrap();
    thread::spawn(move || {
        let client = redis::Client::open("redis://127.0.0.1:6420/2").unwrap();
        let mut con = client.get_connection().unwrap();
        loop {
            let f6 = receiver.recv().unwrap();
            let f6rec = F6Received{
                f6: f6,
                received: Local::now().to_rfc3339(),
            };
            io::redis::push_f6(&mut con, "f6", f6rec);
        }

    });
    process(socket, &sender);

    // let rows: [&[u8]; 2] = [
    //     &[
    //         0x1b, 0x0, 0x41, 0x1, 0x6, 0x4, 0x0, 0x0, 0x0, 0x11, 0x30, 0x30, 0x36, 0x33, 0x32,
    //         0x52, 0x8, 0x30, 0x0, 0x92, 0x9, 0x15, 0x10, 0x0, 0x80, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    //         0x6, 0x32, 0x0, 0x0, 0x0, 0x0, 0x1, 0x25,
    //     ],
    //     &[
    //         0x1b, 0x1, 0x31, 0x1, 0x6, 0x4, 0x0, 0x10, 0x93, 0x59, 0x39, 0x31, 0x31, 0x36, 0x31,
    //         0x36, 0x9, 0x0, 0x0, 0x14, 0x8, 0x66, 0xda, 0x0, 0x8, 0x0, 0x0, 0x0, 0x6, 0x0, 0x0,
    //         0x1, 0x82, 0x0, 0x0, 0x0, 0x0, 0x6, 0x0, 0x0, 0x1, 0x82, 0x0, 0x0, 0x0, 0x0, 0x6, 0x0,
    //         0x0, 0x1, 0x81, 0x0, 0x0, 0x0, 0x0, 0x5, 0x0, 0x0, 0x1, 0x80, 0x0, 0x0, 0x0, 0x0, 0x16,
    //         0x0, 0x0, 0x1, 0x76, 0x0, 0x0, 0x0, 0x0, 0x28, 0x0, 0x0, 0x1, 0x75, 0x0, 0x0, 0x0, 0x0,
    //         0x20, 0x0, 0x0, 0x1, 0x93, 0x0, 0x0, 0x0, 0x0, 0x8, 0x0, 0x0, 0x1, 0x94, 0x0, 0x0, 0x0,
    //         0x0, 0x1, 0x0, 0x0, 0x1, 0x95, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x1, 0x96, 0x0, 0x0,
    //         0x0, 0x0, 0x25, 0x0, 0x0, 0x1, 0x97, 0x0, 0x0, 0x0, 0x0, 0x26, 0xc6,
    //     ],
    // ];
    // for row in rows {
    //     let f6 = bytes2f6(row);
    //     let serialized = serde_json::to_string(&f6).unwrap();
    //     println!("serialized = {}", serialized);
    //     println!("{:?}", bytes2f6(row));
    // }
}
