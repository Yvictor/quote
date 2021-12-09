use crate::paser::bcd;
use serde::{Deserialize, Serialize};
use std::str;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct BidAsk {
    bid_price: [f64; 5],
    bid_volume: [u64; 5],
    ask_price: [f64; 5],
    ask_volume: [u64; 5],
}
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Tick {
    price: f64,
    volume: u64,
}
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Quote {
    bidask: BidAsk,
    tick: Tick,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct F6Header {
    mlen: u8,
    cate: u8,
    fcode: u8,
    fver: u8,
    no: u64,
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

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct F6 {
    header: F6Header,
    quote: Quote,
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
            let r_raw = &ba_raw[(i + 5) * 9..(i + 5 + 1) * 9];
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
        assert_eq!(expected, bytes2f6(input))
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
    ], Quote {
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
    ], Quote {
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
    fn bytes2quote_testcase(input: &[u8], expected: Quote) {
        assert_eq!(expected, bytes2quote(input, 1, 5, 5))
    }
}
