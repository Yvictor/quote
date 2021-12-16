pub mod mcast;
pub mod fs;
pub mod redis;
pub mod mqtt;
use crossbeam_channel::Receiver;
use crate::paser::f6::F6;

pub trait OutProcesser{
    fn recv_f6_process(&mut self, receiver: &Receiver<F6>);
}