use bytes_cast::BytesCast;
use std::fmt;
// use bcd_macro::pbcd2num;
// use num::pow;

const BCD2STR: &[&str] = &[
    "00", "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12", "13", "14", "15",
    "16", "17", "18", "19", "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "30", "31",
    "32", "33", "34", "35", "36", "37", "38", "39", "40", "41", "42", "43", "44", "45", "46", "47",
    "48", "49", "50", "51", "52", "53", "54", "55", "56", "57", "58", "59", "60", "61", "62", "63",
    "64", "65", "66", "67", "68", "69", "70", "71", "72", "73", "74", "75", "76", "77", "78", "79",
    "80", "81", "82", "83", "84", "85", "86", "87", "88", "89", "90", "91", "92", "93", "94", "95",
    "96", "97", "98", "99",
];
const BCD2NUM: &[u64] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 00, 00, 00, 00, 00, 00, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
    00, 00, 00, 00, 00, 00, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 00, 00, 00, 00, 00, 00, 30, 31,
    32, 33, 34, 35, 36, 37, 38, 39, 00, 00, 00, 00, 00, 00, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
    00, 00, 00, 00, 00, 00, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 00, 00, 00, 00, 00, 00, 60, 61,
    62, 63, 64, 65, 66, 67, 68, 69, 00, 00, 00, 00, 00, 00, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79,
    00, 00, 00, 00, 00, 00, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 00, 00, 00, 00, 00, 00, 90, 91,
    92, 93, 94, 95, 96, 97, 98, 99, 00, 00, 00, 00, 00, 00,
];

pub const fn bcd2str(packbcd: u8) -> &'static str {
    let idx: usize = *bcd2num(packbcd) as usize;
    &BCD2STR[idx]
}

pub const fn bcd2num(packbcd: u8) -> &'static u64 {
    &BCD2NUM[packbcd as usize]
}

// TODO const version bcdarr
pub fn bcdarr2num(packbcd_arr: &[u8]) -> u64 {
    let mut num: u64 = 0;
    for packbcd in packbcd_arr {
        num = num * 100 + bcd2num(*packbcd);
    }
    num
}

// macro_rules! pbcd2num {
//     ($n:expr, $i:expr, $arr:ident) => {
//         // println!("{}", n);
//         if ($i > 0) {
//             *bcd2num($arr[$i]) * 10.pow($n - $i) + pbcd2num!($n, $i - 1, $arr)
//         }

//     };
// }

// macro_rules! pbcd2num {
//     ($n:expr, $arr:ident) => {
//         // println!("{}", n);
//         *bcd2num($arr[$n-1]) + pbcd2num!($n-2, $arr)
//     };
// }
// pub const fn packbcd
// pub const fn b2num(n: usize, i: usize, bcd: u8) -> u64 {

// }

// pub struct PackBcd<const N: usize>([u8; N]);
pub const fn packbcd2num<const N: usize>(packbcd: [u8; N], i: usize) -> u64 {
    // println!("{:}", pbcd[0]);
    // pbcd2num!(N, N, packbcd)
    // if (i>0){
    if i > 0 {
        *bcd2num(packbcd[N - i]) * 10_u64.pow(((i - 1) * 2) as u32) + packbcd2num(packbcd, i - 1)
    } else {
        0
    }
    // *bcd2num(packbcd[3])
    //     + *bcd2num(packbcd[2]) * 100
    //     + *bcd2num(packbcd[1]) * 10000
    //     + *bcd2num(packbcd[0]) * 1000000
}

// #[derive(Debug, PartialEq, BytesCast)]
// #[repr(C)]
// pub struct PackBcd<const N: usize>([u8; N]);

// pub fn bcdn2num(packbcd_arr: &[u8; 2]) -> u64 {

// }

#[derive(BytesCast, Debug, PartialEq)]
#[repr(C)]
pub struct TwExMdTimeu6 {
    time: [u8; 6],
}

impl TwExMdTimeu6 {
    pub const fn hour(&self) -> u8 {
        *bcd2num(self.time[0]) as u8
    }
    pub const fn minute(&self) -> u8 {
        *bcd2num(self.time[1]) as u8
    }
    pub const fn second(&self) -> u8 {
        *bcd2num(self.time[2]) as u8
    }
    pub const fn microsecond(&self) -> u64 {
        *bcd2num(self.time[3]) * 10000 + *bcd2num(self.time[4]) * 100 + *bcd2num(self.time[5])
    }
}

impl fmt::Display for TwExMdTimeu6 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02}:{:02}:{:02}.{:06}",
            self.hour(),
            self.minute(),
            self.second(),
            self.microsecond()
        )
    }
}

pub fn bcd2time(packbcd_arr: [u8; 6]) -> String {
    std::format!(
        "{}:{}:{}.{}{}{}",
        bcd2str(packbcd_arr[0]),
        bcd2str(packbcd_arr[1]),
        bcd2str(packbcd_arr[2]),
        bcd2str(packbcd_arr[3]),
        bcd2str(packbcd_arr[4]),
        bcd2str(packbcd_arr[5]),
    )
}

pub fn bcd2price(packbcd_arr: [u8; 5]) -> f64 {
    // 4ns -> 5ns when use ref
    bcdarr2num(&packbcd_arr) as f64 / 10000.
}

pub fn bcd2volume(packbcd_arr: [u8; 4]) -> u64 {
    // 2ns -> 3ns when use ref
    bcdarr2num(&packbcd_arr)
}

#[cfg(test)]
extern crate test_case;

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test]
    fn bcd2str_test() {
        assert_eq!("80", bcd2str(128));
    }

    #[test_case(18, "12"; "0x12 == 18 -> 12")]
    #[test_case(128, "80"; "0x80 == 128 -> 80")]
    fn bcd2str_testcase(input: u8, expected: &str) {
        assert_eq!(expected, bcd2str(input))
    }

    #[test]
    fn bcd2num_test() {
        assert_eq!(80, *bcd2num(128));
    }

    #[test_case(18, 12; "0x12 == 18 -> 12")]
    #[test_case(128, 80; "0x80 == 128 -> 80")]
    fn bcd2num_testcase(input: u8, expected: u64) {
        assert_eq!(expected, *bcd2num(input));
    }

    #[test]
    fn bcd2price_test() {
        assert_eq!(85.2, bcd2price([0, 0, 133, 32, 0]));
    }

    #[test_case([0, 0, 133, 32, 0], 85.2; "0x0, 0x0, 0x85, 0x20, 0x0 -> 85.2")]
    #[test_case([0, 133, 32, 0, 0], 8520.; "0x0, 0x85, 0x20, 0x0, 0x00 -> 8520")]
    fn bcd2price_testcase(input: [u8; 5], expected: f64) {
        assert_eq!(expected, bcd2price(input));
    }

    #[test]
    fn bcd2volume_test() {
        assert_eq!(2, bcd2volume([0, 0, 0, 2]));
        assert_eq!(2020202, bcd2volume([2, 2, 2, 2]));
        assert_eq!(2028520, bcd2volume([2, 2, 133, 32]));
    }

    #[test_case([0, 0, 0, 2], 2; "0, 0, 0, 2 -> 2")]
    #[test_case([0, 0, 133, 32], 8520; "0, 0, 133, 32 -> 8520")]
    fn bcd2volume_testcase(input: [u8; 4], expected: u64) {
        assert_eq!(expected, bcd2volume(input));
    }

    #[test]
    fn packbcd2num_test() {
        assert_eq!(2, packbcd2num([0, 0, 0, 2], 4));
        assert_eq!(8520, packbcd2num([0, 0, 133, 32], 4));
        assert_eq!(28520, packbcd2num([0, 2, 133, 32], 4));
        assert_eq!(2028520, packbcd2num([2, 2, 133, 32], 4));
        assert_eq!(202028520, packbcd2num([2, 2, 2, 133, 32], 5));
    }
}
