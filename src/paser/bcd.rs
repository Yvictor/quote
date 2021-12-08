static BCD2STR: &'static [&str] = &[
    "00", "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12", "13", "14", "15",
    "16", "17", "18", "19", "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "30", "31",
    "32", "33", "34", "35", "36", "37", "38", "39", "40", "41", "42", "43", "44", "45", "46", "47",
    "48", "49", "50", "51", "52", "53", "54", "55", "56", "57", "58", "59", "60", "61", "62", "63",
    "64", "65", "66", "67", "68", "69", "70", "71", "72", "73", "74", "75", "76", "77", "78", "79",
    "80", "81", "82", "83", "84", "85", "86", "87", "88", "89", "90", "91", "92", "93", "94", "95",
    "96", "97", "98", "99",
];
static BCD2NUM: &'static [u64] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 00, 00, 00, 00, 00, 00, 
    10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 00, 00, 00, 00, 00, 00, 
    20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 00, 00, 00, 00, 00, 00, 
    30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 00, 00, 00, 00, 00, 00, 
    40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 00, 00, 00, 00, 00, 00, 
    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 00, 00, 00, 00, 00, 00, 
    60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 00, 00, 00, 00, 00, 00, 
    70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 00, 00, 00, 00, 00, 00, 
    80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 00, 00, 00, 00, 00, 00, 
    90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 00, 00, 00, 00, 00, 00,
];

pub fn bcd2str(packbcd: u8) -> &'static str {
    let idx: usize = *bcd2num(packbcd) as usize;
    &BCD2STR[idx]
}

pub fn bcd2num(packbcd: u8) -> &'static u64 {
    &BCD2NUM[packbcd as usize]
}

pub fn bcdarr2num(packbcd_arr: &[u8]) -> u64 {
    let mut num: u64 = 0;
    for packbcd in packbcd_arr {
        num = num * 100 + bcd2num(*packbcd);
    }
    num
}

pub fn price2long(packbcd_arr: [u8; 5]) -> f64 {
    // 4ns -> 5ns when use ref
    bcdarr2num(&packbcd_arr) as f64 / 10000.
}

pub fn volume2long(packbcd_arr: [u8; 4]) -> u64 {
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
    fn price2long_test() {
        assert_eq!(85.2, price2long([0, 0, 133, 32, 0]));
    }

    #[test_case([0, 0, 133, 32, 0], 85.2; "0x0, 0x0, 0x85, 0x20, 0x0 -> 85.2")]
    #[test_case([0, 133, 32, 0, 0], 8520.; "0x0, 0x85, 0x20, 0x0, 0x00 -> 8520")]
    fn price2long_testcase(input: [u8; 5], expected: f64) {
        assert_eq!(expected, price2long(input));
    }

    #[test]
    fn volume2long_test() {
        assert_eq!(2, volume2long([0, 0, 0, 2]));
    }

    #[test_case([0, 0, 0, 2], 2; "0, 0, 0, 2 -> 2")]
    #[test_case([0, 0, 133, 32], 8520; "0, 0, 133, 32 -> 8520")]
    fn volume2long_testcase(input: [u8; 4], expected: u64) {
        assert_eq!(expected, volume2long(input));
    }
}
