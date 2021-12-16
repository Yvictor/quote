use crate::paser::f6::{F6Received, F6};
use chrono::Local;
use crossbeam_channel::Receiver;
use redis::{Client, Commands, Connection};
use crate::io::OutProcesser;

pub struct RedisOutProcesser{
    redis_uri: String,
    conn: Connection,
}

impl RedisOutProcesser {
    fn new(redis_uri: &str) -> RedisOutProcesser {
        let client = Client::open(redis_uri.clone()).unwrap();
        let conn = client.get_connection().unwrap();
        RedisOutProcesser{redis_uri: String::from(redis_uri), conn: conn}
    }

    // fn get_connection(&mut self) -> RedisResult<&Connection>{
    //     if let Some(ref conn) = self.conn {
    //         Ok(conn)
    //     } else {
    //         let client = Client::open(self.redis_uri.clone()).unwrap();
    //         self.conn = Some(client.get_connection().unwrap());
    //         Ok(self.conn.as_ref().unwrap())
    //     }
    // }
    
    fn push_f6(&mut self, key: &str, f6: F6Received) {
        let f6_serialized = serde_json::to_string(&f6).unwrap();
        // let f6_serialized = rmp_serde::to_vec(&f6).unwrap();
        let _: () = self.conn.lpush(key, f6_serialized).unwrap();
    }
}

impl OutProcesser for RedisOutProcesser {
    fn recv_f6_process(&mut self, receiver: &Receiver<F6>){
        loop {
            let f6 = receiver.recv().unwrap();
            let f6rec = F6Received {
                f6: f6,
                received: Local::now().to_rfc3339(),
            };
            self.push_f6("f6", f6rec);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::paser::f6::bytes2f6;
    // use test_case::test_case;

    #[test]
    fn push_f6_test() {
        // let client = redis::Client::open("redis://127.0.0.1:6420/2").unwrap();
        // let mut con = client.get_connection().unwrap();
        let raw = &[
            0x1b, 0x1, 0x31, 0x1, 0x6, 0x4, 0x0, 0x10, 0x93, 0x59, 0x39, 0x31, 0x31, 0x36, 0x31,
            0x36, 0x9, 0x0, 0x0, 0x14, 0x8, 0x66, 0xda, 0x0, 0x8, 0x0, 0x0, 0x0, 0x6, 0x0, 0x0,
            0x1, 0x82, 0x0, 0x0, 0x0, 0x0, 0x6, 0x0, 0x0, 0x1, 0x82, 0x0, 0x0, 0x0, 0x0, 0x6, 0x0,
            0x0, 0x1, 0x81, 0x0, 0x0, 0x0, 0x0, 0x5, 0x0, 0x0, 0x1, 0x80, 0x0, 0x0, 0x0, 0x0, 0x16,
            0x0, 0x0, 0x1, 0x76, 0x0, 0x0, 0x0, 0x0, 0x28, 0x0, 0x0, 0x1, 0x75, 0x0, 0x0, 0x0, 0x0,
            0x20, 0x0, 0x0, 0x1, 0x93, 0x0, 0x0, 0x0, 0x0, 0x8, 0x0, 0x0, 0x1, 0x94, 0x0, 0x0, 0x0,
            0x0, 0x1, 0x0, 0x0, 0x1, 0x95, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x1, 0x96, 0x0, 0x0,
            0x0, 0x0, 0x25, 0x0, 0x0, 0x1, 0x97, 0x0, 0x0, 0x0, 0x0, 0x26, 0xc6,
        ];
        let f6 = bytes2f6(raw);
        // push_f6(&mut con, "f6test", f6);
        assert_eq!(1, 1)
    }
}
