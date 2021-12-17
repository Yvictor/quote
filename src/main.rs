// use quote::paser::f6::bytes2f6;
// use quote::io::fs::{readf6file, readf6filebuffer};
use quote::io;
use quote::io::mcast::{join_mcast, process};
use quote::io::{OutProcesser};
use quote::paser::f6::F6;
use quote::utils::{getenv, setup_log, str2ip};
use std::net::SocketAddr;
use std::thread;
// use std::path::Path;
// use std::sync::mpsc::{channel, Sender, Receiver};
use crossbeam_channel::{bounded, Receiver, Sender};

#[macro_use]
extern crate lazy_static;

lazy_static! {
    pub static ref MCAST_ADDR: SocketAddr = str2ip(&getenv("MCAST_GROUP", "224.0.100.100:10000"));
    pub static ref MCAST_IF_ADDR: SocketAddr =
        str2ip(&getenv("MCAST_IF_ADDR", "192.168.32.23:10000"));
    pub static ref REDIS_URI: String = getenv("REDIS_URI", "redis://127.0.0.1:6420/2");
}

fn main() {
    setup_log();
    // let (sender, receiver): (Sender<F6>, Receiver<F6>) = channel();
    let (sender, receiver): (Sender<F6>, Receiver<F6>) = bounded(4096);

    let mut redis_outp = io::redis::RedisOutProcesser::new(&REDIS_URI);
    let socket = join_mcast(&MCAST_ADDR, &MCAST_IF_ADDR).unwrap();
    thread::spawn(move || redis_outp.recv_f6_process(&receiver));
    process(socket, &sender);

    // let path = Path::new("tests/data/f6_01000001_01001000_TP03.new");
    // // let path = Path::new("集中市場行情格式六_04000001_04500000_TP09.new");
    // let display = path.display();
    // log::info!("start parsing file: {}", display);
    // // readf6file(&Path::new("集中市場行情格式六_01000001_01500000_TP03.new"), f6handler);
    // readf6file(&path, f6handler);
    // log::info!("readf6file");
    // readf6filebuffer(&path, f6handler);
    // log::info!("finish");
}
