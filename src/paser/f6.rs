use crate::paser::bcd;
use serde::{Deserialize, Serialize};
use std::str;
// use chrono::prelude::{Local};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct BidAsk {
    bid_price: [f64; 5],
    bid_volume: [u64; 5],
    ask_price: [f64; 5],
    ask_volume: [u64; 5],
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Tick {
    price: f64,
    volume: u64,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Quote {
    bidask: BidAsk,
    tick: Tick,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct F6Header {
    pub mlen: u8,
    cate: u8,
    fcode: u8,
    fver: u8,
    pub no: u64,
    symbol: String,
    time: String,
    n_match: u8,
    n_bid: u8,
    n_ask: u8,
    trice: u8,
    simulation: bool,
    delay_open: bool,
    dalay_close: bool,
    auction: bool,
    opened: bool,
    closed: bool,
    volsum: u64,
}

impl F6Header {
    pub fn n_info(&self) -> (usize, usize, usize) {
        (*&self.n_match as usize, *&self.n_bid as usize, *&self.n_ask as usize)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct F6 {
    pub header: F6Header,
    pub quote: Quote,
    // pub received: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct F6Received {
    pub f6: F6,
    pub received: String,
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct Rawf6Fixed {
    esc_code: u8,
    mlen: [u8; 2],
    cate: u8,
    fcode: u8,
    fver: u8,
    no: [u8; 4],
    symbol: [u8; 6],
    time: [u8; 6],
    bmp: u8,
    ud: u8,
    st: u8,
    volsum: [u8; 4],
}

pub fn bytes2mlen(raw: &[u8]) -> usize {
    bcd::bcdarr2num(&raw[1..3]) as usize
}

pub fn bytes2fcode(raw: &[u8]) -> &'static u64 {
    bcd::bcd2num(raw[4])
}

pub fn bytes2header(raw: &[u8]) -> F6Header {
    let fixed = Rawf6Fixed {
        esc_code: raw[0],
        mlen: raw[1..3].try_into().unwrap(),
        cate: raw[3],
        fcode: raw[4],
        fver: raw[5],
        no: raw[6..10].try_into().unwrap(),
        symbol: raw[10..16].try_into().unwrap(),
        time: raw[16..22].try_into().unwrap(),
        bmp: raw[22],
        ud: raw[23],
        st: raw[24],
        volsum: raw[25..29].try_into().unwrap(),
    };
    // println!("{:?}", fixed);
    let header = F6Header {
        mlen: bcd::bcdarr2num(&fixed.mlen) as u8,
        cate: *bcd::bcd2num(fixed.cate) as u8,
        fcode: *bcd::bcd2num(fixed.fcode) as u8,
        fver: *bcd::bcd2num(fixed.fver) as u8,
        no: bcd::bcdarr2num(&fixed.no),
        symbol: String::from(str::from_utf8(&fixed.symbol).unwrap()),
        time: bcd::bcd2time(fixed.time),
        n_match: (fixed.bmp & 0x80) >> 7,
        n_bid: (fixed.bmp & 0x70) >> 4,
        n_ask: (fixed.bmp & 0x0E) >> 1,
        trice: (fixed.ud & 0x03),
        simulation: (fixed.st & 0x80) != 0,
        delay_open: (fixed.st & 0x40) != 0,
        dalay_close: (fixed.st & 0x20) != 0,
        auction: (fixed.st & 0x10) != 0,
        opened: (fixed.st & 0x08) != 0,
        closed: (fixed.st & 0x04) != 0,
        volsum: bcd::bcdarr2num(&fixed.volsum),
    };
    // println!("{:?}", header);
    header
}

pub fn bytes2quote(packbcd_arr: &[u8], n_match: usize, n_bid: usize, n_ask: usize) -> Quote {
    let mut tick_price = 0.0;
    let mut tick_volume = 0;
    if n_match > 0 {
        let tick_raw: &[u8; 9] = &packbcd_arr[..9 * n_match].try_into().unwrap();
        tick_price = bcd::bcd2price(tick_raw[..5].try_into().unwrap());
        tick_volume = bcd::bcd2volume(tick_raw[5..9].try_into().unwrap());
    }
    let ba_raw: &[u8] = &packbcd_arr[9 * n_match..];
    let mut bid_price: [f64; 5] = [0.; 5];
    let mut bid_volume: [u64; 5] = [0; 5];
    let mut ask_price: [f64; 5] = [0.; 5];
    let mut ask_volume: [u64; 5] = [0; 5];
    for i in 0..5 {
        if n_bid > i {
            let r_raw = &ba_raw[i * 9..(i + 1) * 9];
            bid_price[i] = bcd::bcd2price(r_raw[..5].try_into().unwrap());
            bid_volume[i] = bcd::bcd2volume(r_raw[5..9].try_into().unwrap());
        }
        if n_ask > i {
            let r_raw = &ba_raw[(i + n_bid) * 9..(i + n_bid + 1) * 9];
            ask_price[i] = bcd::bcd2price(r_raw[..5].try_into().unwrap());
            ask_volume[i] = bcd::bcd2volume(r_raw[5..9].try_into().unwrap());
        }
    }
    Quote {
        bidask: BidAsk {
            bid_price,
            bid_volume,
            ask_price,
            ask_volume,
        },
        tick: Tick {
            price: tick_price,
            volume: tick_volume,
        },
    }
}

pub fn bytes2f6(raw: &[u8]) -> F6 {
    let header = bytes2header(raw);
    let (n_match, n_bid, n_ask) = (
        header.n_match as usize,
        header.n_bid as usize,
        header.n_ask as usize,
    );
    let tn = n_match + n_bid + n_ask;
    F6 {
        header: header,
        quote: bytes2quote(&raw[29..29 + 9 * (tn)], n_match, n_bid, n_ask),
        // received: Local::now().to_rfc3339(),
    }
}

#[cfg(test)]
extern crate test_case;

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test]
    fn bytes2f6_test() {
        assert_eq!(
            bytes2f6(&[
                0x1b, 0x1, 0x31, 0x1, 0x6, 0x4, 0x0, 0x10, 0x93, 0x59, 0x39, 0x31, 0x31, 0x36,
                0x31, 0x36, 0x9, 0x0, 0x0, 0x14, 0x8, 0x66, 0xda, 0x0, 0x8, 0x0, 0x0, 0x0, 0x6,
                0x0, 0x0, 0x1, 0x82, 0x0, 0x0, 0x0, 0x0, 0x6, 0x0, 0x0, 0x1, 0x82, 0x0, 0x0, 0x0,
                0x0, 0x6, 0x0, 0x0, 0x1, 0x81, 0x0, 0x0, 0x0, 0x0, 0x5, 0x0, 0x0, 0x1, 0x80, 0x0,
                0x0, 0x0, 0x0, 0x16, 0x0, 0x0, 0x1, 0x76, 0x0, 0x0, 0x0, 0x0, 0x28, 0x0, 0x0, 0x1,
                0x75, 0x0, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x1, 0x93, 0x0, 0x0, 0x0, 0x0, 0x8, 0x0,
                0x0, 0x1, 0x94, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x1, 0x95, 0x0, 0x0, 0x0, 0x0,
                0x1, 0x0, 0x0, 0x1, 0x96, 0x0, 0x0, 0x0, 0x0, 0x25, 0x0, 0x0, 0x1, 0x97, 0x0, 0x0,
                0x0, 0x0, 0x26, 0xc6,
            ]),
            F6 {
                header: F6Header {
                    mlen: 131,
                    cate: 1,
                    fcode: 6,
                    fver: 4,
                    no: 109359,
                    symbol: String::from("911616"),
                    time: String::from("09:00:00.140866"),
                    n_match: 1,
                    n_bid: 5,
                    n_ask: 5,
                    trice: 0,
                    simulation: false,
                    delay_open: false,
                    dalay_close: false,
                    auction: false,
                    opened: true,
                    closed: false,
                    volsum: 6
                },
                quote: Quote {
                    bidask: BidAsk {
                        bid_price: [1.82, 1.81, 1.8, 1.76, 1.75],
                        bid_volume: [6, 5, 16, 28, 20],
                        ask_price: [1.93, 1.94, 1.95, 1.96, 1.97],
                        ask_volume: [8, 1, 1, 25, 26],
                    },
                    tick: Tick {
                        price: 1.82,
                        volume: 6,
                    },
                }
            }
        )
    }

    #[test_case(&[
        0x1b, 0x0, 0x41, 0x1, 0x6, 0x4, 0x0, 0x0, 0x0, 0x11, 0x30, 0x30, 0x36, 0x33, 0x32,
        0x52, 0x8, 0x30, 0x0, 0x92, 0x9, 0x15, 0x10, 0x0, 0x80, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        0x6, 0x32, 0x0, 0x0, 0x0, 0x0, 0x1, 0x25,
    ], F6{header: F6Header {
        mlen: 41,
        cate: 1,
        fcode: 6,
        fver: 4,
        no: 11,
        symbol: String::from("00632R"),
        time: String::from("08:30:00.920915"),
        n_match: 0,
        n_bid: 1,
        n_ask: 0,
        trice: 0,
        simulation: true,
        delay_open: false,
        dalay_close: false,
        auction: false,
        opened: false,
        closed: false,
        volsum: 0
        },
        quote: Quote {
            bidask: BidAsk {
                bid_price: [6.32, 0.0, 0.0, 0.0, 0.0],
                bid_volume: [1, 0, 0, 0, 0],
                ask_price: [0.0, 0.0, 0.0, 0.0, 0.0],
                ask_volume: [0, 0, 0, 0, 0],
            },
            tick: Tick {
                price: 0.0,
                volume: 0,
            },
        }
    }; "case bid only1")]
    fn bytes2f6_testcase(input: &[u8], expected: F6) {
        //[27, 1, 20, 1, 1, 9, 0, 0, 0, 1, 48, 48, 53, 48, 32, 32, 164, 184, 164, 106, 165, 120, 198, 87, 53, 48, 32, 32, 32, 32, 32, 32, 48, 48, 32, 32, 32, 32, 0, 48, 0, 1, 65, 149, 0, 0, 1, 86, 16, 0, 0, 1, 39, 128, 0, 32, 32, 32, 65, 89, 89, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32, 0, 16, 0, 32, 32, 32, 1, 215, 13, 10]
        //[27, 1, 20, 1, 1, 9, 0, 1, 53, 0, 48, 52, 55, 51, 52, 54, 165, 120, 164, 198, 176, 234, 178, 188, 49, 53, 193, 202, 48, 49, 32, 32, 48, 48, 87, 50, 32, 32, 0, 48, 0, 0, 0, 86, 0, 0, 0, 1, 118, 0, 0, 0, 0, 1, 0, 32, 32, 32, 32, 32, 32, 0, 0, 0, 89, 0, 0, 146, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 37, 0, 0, 1, 80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32, 34, 5, 4, 32, 0, 16, 0, 32, 32, 32, 2, 20, 13, 10]
        //[27, 0, 134, 1, 6, 4, 0, 5, 18, 151, 51, 48, 52, 54, 32, 32, 8, 72, 85, 73, 134, 5, 208, 32, 128, 0, 0, 0, 0, 0, 0, 72, 5, 0, 0, 0, 3, 55, 0, 0, 72, 5, 0, 0, 0, 20, 81, 0, 0, 71, 85, 0, 0, 0, 0, 1, 0, 0, 71, 0, 0, 0, 0, 0, 18, 0, 0, 70, 0, 0, 0, 0, 0, 23, 0, 0, 69, 144, 0, 0, 0, 0, 3, 27, 13, 10] 
        //[27, 1, 49, 1, 6, 4, 0, 5, 18, 152, 50, 52, 57, 56, 32, 32, 8, 72, 85, 73, 134, 5, 218, 0, 128, 0, 0, 0, 0, 0, 0, 128, 48, 0, 0, 0, 2, 119, 0, 0, 128, 48, 0, 0, 0, 0, 135, 0, 0, 128, 32, 0, 0, 0, 1, 41, 0, 0, 128, 16, 0, 0, 0, 1, 0, 0, 0, 128, 0, 0, 0, 0, 0, 121, 0, 0, 121, 144, 0, 0, 0, 0, 9, 0, 0, 128, 64, 0, 0, 0, 0, 8, 0, 0, 128, 112, 0, 0, 0, 1, 32, 0, 0, 128, 128, 0, 0, 0, 0, 37, 0, 0, 128, 144, 0, 0, 0, 0, 2, 0, 0, 129, 0, 0, 0, 0, 0, 20, 119, 13, 10]
        assert_eq!(expected, bytes2f6(input))
    }

    #[test]
    fn bytes2mlen_test() {
        assert_eq!(
            bytes2mlen(&[
                0x1b, 0x1, 0x31, 0x1, 0x6, 0x4, 0x0, 0x10, 0x93, 0x59, 0x39, 0x31, 0x31, 0x36,
                0x31, 0x36, 0x9, 0x0, 0x0, 0x14, 0x8, 0x66, 0xda, 0x0, 0x8, 0x0, 0x0, 0x0, 0x6,
                0x0, 0x0, 0x1, 0x82, 0x0, 0x0, 0x0, 0x0, 0x6, 0x0, 0x0, 0x1, 0x82, 0x0, 0x0, 0x0,
                0x0, 0x6, 0x0, 0x0, 0x1, 0x81, 0x0, 0x0, 0x0, 0x0, 0x5, 0x0, 0x0, 0x1, 0x80, 0x0,
                0x0, 0x0, 0x0, 0x16, 0x0, 0x0, 0x1, 0x76, 0x0, 0x0, 0x0, 0x0, 0x28, 0x0, 0x0, 0x1,
                0x75, 0x0, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x1, 0x93, 0x0, 0x0, 0x0, 0x0, 0x8, 0x0,
                0x0, 0x1, 0x94, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x1, 0x95, 0x0, 0x0, 0x0, 0x0,
                0x1, 0x0, 0x0, 0x1, 0x96, 0x0, 0x0, 0x0, 0x0, 0x25, 0x0, 0x0, 0x1, 0x97, 0x0, 0x0,
                0x0, 0x0, 0x26, 0xc6,
            ]),
            131
        )
    }

    #[test_case(&[
        0x1b, 0x1, 0x31, 0x1, 0x6, 0x4, 0x0, 0x10, 0x93, 0x59, 0x39, 0x31, 0x31, 0x36,
        0x31, 0x36, 0x9, 0x0, 0x0, 0x14, 0x8, 0x66, 0xda, 0x0, 0x8, 0x0, 0x0, 0x0, 0x6,
        0x0, 0x0, 0x1, 0x82, 0x0, 0x0, 0x0, 0x0, 0x6, 0x0, 0x0, 0x1, 0x82, 0x0, 0x0, 0x0,
        0x0, 0x6, 0x0, 0x0, 0x1, 0x81, 0x0, 0x0, 0x0, 0x0, 0x5, 0x0, 0x0, 0x1, 0x80, 0x0,
        0x0, 0x0, 0x0, 0x16, 0x0, 0x0, 0x1, 0x76, 0x0, 0x0, 0x0, 0x0, 0x28, 0x0, 0x0, 0x1,
        0x75, 0x0, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x1, 0x93, 0x0, 0x0, 0x0, 0x0, 0x8, 0x0,
        0x0, 0x1, 0x94, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x1, 0x95, 0x0, 0x0, 0x0, 0x0,
        0x1, 0x0, 0x0, 0x1, 0x96, 0x0, 0x0, 0x0, 0x0, 0x25, 0x0, 0x0, 0x1, 0x97, 0x0, 0x0,
        0x0, 0x0, 0x26, 0xc6,
    ], 131; "case1")]
    #[test_case(&[
        27, 1, 20, 1, 1, 9, 0, 0, 0, 1, 48, 48, 53, 48, 32, 32, 164, 184, 164, 106, 165, 120, 198, 
        87, 53, 48, 32, 32, 32, 32, 32, 32, 48, 48, 32, 32, 32, 32, 0, 48, 0, 1, 65, 149, 0, 0, 1
        , 86, 16, 0, 0, 1, 39, 128, 0, 32, 32, 32, 65, 89, 89, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
        32, 0, 16, 0, 32, 32, 32, 1, 215, 13, 10
    ], 114; "case2")]
    #[test_case(&[
        27, 1, 20, 1, 1, 9, 0, 0, 0, 1, 48, 51, 52, 51, 48, 80, 184, 85, 174, 252, 164, 164, 171, 72, 
        49, 55, 176, 226, 48, 49, 32, 32, 48, 48, 87, 52, 32, 32, 0, 48, 0, 0, 1, 8, 0, 0, 0, 1, 84, 
        0, 0, 0, 0, 98, 0, 32, 32, 32, 32, 32, 32, 0, 0, 0, 89, 0, 1, 152, 136, 0, 0, 0, 0, 0, 0, 0, 
        0, 0, 0, 0, 0, 0, 0, 80, 0, 0, 0, 34, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32, 34, 7, 18, 32, 0, 
        16, 0, 32, 32, 32, 2, 34, 13, 10
    ], 114; "case3")]
    #[test_case(&[
        27, 3, 116, 1, 18, 3, 0, 0, 0, 1, 16, 48, 48, 53, 48, 32, 32, 0, 1, 66, 32, 0, 0, 1, 66, 69, 
        0, 0, 1, 65, 112, 0, 0, 1, 65, 128, 0, 0, 0, 67, 145, 19, 36, 84, 146, 67, 7, 48, 48, 53, 49,
         32, 32, 0, 0, 96, 0, 0, 0, 0, 96, 21, 0, 0, 0, 89, 149, 0, 0, 0, 89, 149, 0, 0, 0, 0, 64, 19, 
         16, 23, 120, 151, 18, 48, 48, 53, 50, 32, 32, 0, 1, 48, 117, 0, 0, 1, 49, 0, 0, 0, 1, 48, 80, 
         0, 0, 1, 48, 96, 0, 0, 0, 3, 9, 19, 24, 53, 8, 149, 152, 48, 48, 53, 51, 32, 32, 0, 0, 104, 16, 
         0, 0, 0, 104, 32, 0, 0, 0, 104, 0, 0, 0, 0, 104, 16, 0, 0, 0, 0, 8, 19, 6, 38, 55, 49, 36, 48, 
         48, 53, 52, 32, 32, 0, 0, 48, 153, 0, 0, 0, 49, 4, 0, 0, 0, 48, 134, 0, 0, 0, 49, 4, 0, 0, 0, 0, 
         36, 18, 48, 0, 101, 100, 149, 48, 48, 53, 53, 32, 32, 0, 0, 36, 64, 0, 0, 0, 36, 72, 0, 0, 0, 36, 
         48, 0, 0, 0, 36, 48, 0, 0, 0, 7, 52, 19, 36, 69, 24, 117, 129, 48, 48, 53, 54, 32, 32, 0, 0, 51, 
         50, 0, 0, 0, 51, 69, 0, 0, 0, 51, 50, 0, 0, 0, 51, 51, 0, 0, 1, 3, 151, 19, 36, 86, 150, 89, 40, 
         48, 48, 53, 55, 32, 32, 0, 0, 152, 37, 0, 0, 0, 152, 37, 0, 0, 0, 151, 112, 0, 0, 0, 151, 149, 0,
          0, 0, 0, 37, 18, 48, 0, 148, 6, 68, 48, 48, 54, 49, 32, 32, 0, 0, 35, 87, 0, 0, 0, 35, 130, 0, 0, 
          0, 35, 80, 0, 0, 0, 35, 114, 0, 0, 0, 3, 21, 19, 36, 49, 3, 148, 8, 48, 48, 54, 50, 48, 51, 0, 0, 
          104, 0, 0, 0, 0, 104, 64, 0, 0, 0, 103, 144, 0, 0, 0, 104, 64, 0, 0, 0, 0, 3, 18, 48, 2, 50, 148, 
          6, 105, 13, 10
    ], 374; "case4")]
    fn bytes2mlen_testcase(input: &[u8], expected: usize) {
        assert_eq!(bytes2mlen(input), expected)
    }

    #[test]
    fn bytes2header_test() {
        assert_eq!(
            bytes2header(&[
                0x1b, 0x1, 0x31, 0x1, 0x6, 0x4, 0x0, 0x10, 0x93, 0x59, 0x39, 0x31, 0x31, 0x36,
                0x31, 0x36, 0x9, 0x0, 0x0, 0x14, 0x8, 0x66, 0xda, 0x0, 0x8, 0x0, 0x0, 0x0, 0x6,
                0x0, 0x0, 0x1, 0x82, 0x0, 0x0, 0x0, 0x0, 0x6, 0x0, 0x0, 0x1, 0x82, 0x0, 0x0, 0x0,
                0x0, 0x6, 0x0, 0x0, 0x1, 0x81, 0x0, 0x0, 0x0, 0x0, 0x5, 0x0, 0x0, 0x1, 0x80, 0x0,
                0x0, 0x0, 0x0, 0x16, 0x0, 0x0, 0x1, 0x76, 0x0, 0x0, 0x0, 0x0, 0x28, 0x0, 0x0, 0x1,
                0x75, 0x0, 0x0, 0x0, 0x0, 0x20, 0x0, 0x0, 0x1, 0x93, 0x0, 0x0, 0x0, 0x0, 0x8, 0x0,
                0x0, 0x1, 0x94, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x1, 0x95, 0x0, 0x0, 0x0, 0x0,
                0x1, 0x0, 0x0, 0x1, 0x96, 0x0, 0x0, 0x0, 0x0, 0x25, 0x0, 0x0, 0x1, 0x97, 0x0, 0x0,
                0x0, 0x0, 0x26, 0xc6,
            ]),
            F6Header {
                mlen: 131,
                cate: 1,
                fcode: 6,
                fver: 4,
                no: 109359,
                symbol: String::from("911616"),
                time: String::from("09:00:00.140866"),
                n_match: 1,
                n_bid: 5,
                n_ask: 5,
                trice: 0,
                simulation: false,
                delay_open: false,
                dalay_close: false,
                auction: false,
                opened: true,
                closed: false,
                volsum: 6
            }
        )
    }

    #[test]
    fn f6header_n_info_test() {
        let f6 = F6Header {
            mlen: 131,
            cate: 1,
            fcode: 6,
            fver: 4,
            no: 109359,
            symbol: String::from("911616"),
            time: String::from("09:00:00.140866"),
            n_match: 1,
            n_bid: 5,
            n_ask: 5,
            trice: 0,
            simulation: false,
            delay_open: false,
            dalay_close: false,
            auction: false,
            opened: true,
            closed: false,
            volsum: 6
        };
        assert_eq!((1, 5, 5), f6.n_info())
    }

    #[test_case(&[
        0x1b, 0x0, 0x41, 0x1, 0x6, 0x4, 0x0, 0x0, 0x0, 0x11, 0x30, 0x30, 0x36, 0x33, 0x32,
        0x52, 0x8, 0x30, 0x0, 0x92, 0x9, 0x15, 0x10, 0x0, 0x80, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        0x6, 0x32, 0x0, 0x0, 0x0, 0x0, 0x1, 0x25,
    ], F6Header {
        mlen: 41,
        cate: 1,
        fcode: 6,
        fver: 4,
        no: 11,
        symbol: String::from("00632R"),
        time: String::from("08:30:00.920915"),
        n_match: 0,
        n_bid: 1,
        n_ask: 0,
        trice: 0,
        simulation: true,
        delay_open: false,
        dalay_close: false,
        auction: false,
        opened: false,
        closed: false,
        volsum: 0
    }; "case bid only1")]
    fn bytes2header_testcase(input: &[u8], expected: F6Header) {
        assert_eq!(expected, bytes2header(input))
    }

    #[test]
    fn bytes2quote_test() {
        assert_eq!(
            Quote {
                bidask: BidAsk {
                    bid_price: [545.0, 541.0, 540.0, 530.0, 522.0],
                    bid_volume: [1, 1, 1, 2, 24],
                    ask_price: [555.0, 558.0, 560.0, 561.0, 562.0],
                    ask_volume: [1, 1, 1, 2, 1],
                },
                tick: Tick {
                    price: 552.0,
                    volume: 2,
                },
            },
            bytes2quote(
                &[
                    0x0, 0x5, 0x52, 0x0, 0x0, 0x0, 0x0, 0x0, 0x2, 0x0, 0x5, 0x45, 0x0, 0x0, 0x0,
                    0x0, 0x0, 0x1, 0x0, 0x5, 0x41, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x5, 0x40,
                    0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x5, 0x30, 0x0, 0x0, 0x0, 0x0, 0x0, 0x2,
                    0x0, 0x5, 0x22, 0x0, 0x0, 0x0, 0x0, 0x0, 0x24, 0x0, 0x5, 0x55, 0x0, 0x0, 0x0,
                    0x0, 0x0, 0x1, 0x0, 0x5, 0x58, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x5, 0x60,
                    0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x5, 0x61, 0x0, 0x0, 0x0, 0x0, 0x0, 0x2,
                    0x0, 0x5, 0x62, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1
                ],
                1,
                5,
                5
            )
        );
    }

    #[test_case(&[
        0x0, 0x5, 0x52, 0x0, 0x0, 0x0, 0x0, 0x0, 0x2, 0x0, 0x5, 0x45, 0x0, 0x0, 0x0, 0x0,
        0x0, 0x1, 0x0, 0x5, 0x41, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x5, 0x40, 0x0, 0x0,
        0x0, 0x0, 0x0, 0x1, 0x0, 0x5, 0x30, 0x0, 0x0, 0x0, 0x0, 0x0, 0x2, 0x0, 0x5, 0x22,
        0x0, 0x0, 0x0, 0x0, 0x0, 0x24, 0x0, 0x5, 0x55, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0,
        0x5, 0x58, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x5, 0x60, 0x0, 0x0, 0x0, 0x0, 0x0,
        0x1, 0x0, 0x5, 0x61, 0x0, 0x0, 0x0, 0x0, 0x0, 0x2, 0x0, 0x5, 0x62, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x1
    ], 1, 5, 5, Quote {
        bidask: BidAsk {
            bid_price: [545.0, 541.0, 540.0, 530.0, 522.0],
            bid_volume: [1, 1, 1, 2, 24],
            ask_price: [555.0, 558.0, 560.0, 561.0, 562.0],
            ask_volume: [1, 1, 1, 2, 1],
        },
        tick: Tick {
            price: 552.0,
            volume: 2,
        },
    }; "case1")]
    #[test_case(&[
        0x0, 0x0, 0x1, 0x82, 0x0, 0x0, 0x0, 0x0, 0x6, 0x0, 0x0, 0x1, 0x82, 0x0, 0x0, 0x0, 0x0, 0x6,
        0x0, 0x0, 0x1, 0x81, 0x0, 0x0, 0x0, 0x0, 0x5, 0x0, 0x0, 0x1, 0x80, 0x0, 0x0, 0x0, 0x0,
        0x16, 0x0, 0x0, 0x1, 0x76, 0x0, 0x0, 0x0, 0x0, 0x28, 0x0, 0x0, 0x1, 0x75, 0x0, 0x0, 0x0,
        0x0, 0x20, 0x0, 0x0, 0x1, 0x93, 0x0, 0x0, 0x0, 0x0, 0x8, 0x0, 0x0, 0x1, 0x94, 0x0, 0x0,
        0x0, 0x0, 0x1, 0x0, 0x0, 0x1, 0x95, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x1, 0x96, 0x0, 0x0,
        0x0, 0x0, 0x25, 0x0, 0x0, 0x1, 0x97, 0x0, 0x0, 0x0, 0x0, 0x26,
    ], 1, 5, 5, Quote {
        bidask: BidAsk {
            bid_price: [1.82, 1.81, 1.8, 1.76, 1.75],
            bid_volume: [6, 5, 16, 28, 20],
            ask_price: [1.93, 1.94, 1.95, 1.96, 1.97],
            ask_volume: [8, 1, 1, 25, 26],
        },
        tick: Tick {
            price: 1.82,
            volume: 6,
        },
    }; "case2")]
    #[test_case(&[
        0, 0, 7, 6, 0, 0, 0, 0, 16, 0, 0, 7, 5, 0, 0, 0, 0, 16, 0, 0, 6, 128, 0, 0, 0, 0, 2, 0, 0, 6,
        53, 0, 0, 0, 0, 4, 0, 0, 7, 7, 0, 0, 0, 0, 16, 0, 0, 7, 8, 0, 0, 0, 0, 16, 0, 0, 7, 38, 0, 0,
        0, 0, 64, 181, 13, 10,
    ], 0, 3, 4, Quote {
        bidask: BidAsk {
            bid_price: [7.06, 7.05, 6.8, 0.0, 0.0], 
            bid_volume: [10, 10, 2, 0, 0], 
            ask_price: [6.35, 7.07, 7.08, 7.26, 0.0], 
            ask_volume: [4, 10, 10, 40, 0],
        },
        tick: Tick {
            price: 0.0, volume: 0
        },
    }; "case3")]
    fn bytes2quote_testcase(
        input: &[u8],
        n_match: usize,
        n_bid: usize,
        n_ask: usize,
        expected: Quote,
    ) {
        assert_eq!(expected, bytes2quote(input, n_match, n_bid, n_ask))
    }
}
