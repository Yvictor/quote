use crate::paser::bcd;
use std::str;

use bytes_cast::BytesCast;

#[derive(BytesCast, Debug, PartialEq)]
#[repr(C)]
pub struct TWExMdHead {
    // Tw Exchange Market data head
    esc_code: u8,
    len: [u8; 2],
    market: u8,
    fcode: u8,
    fver: u8,
    seqno: [u8; 4],
}

enum TwMarket {
    TWSE,
    TPEX,
}

impl TWExMdHead {
    pub fn len(&self) -> u64 {
        bcd::bcdarr2num(&self.len)
    }
    pub const fn market(&self) -> u8 {
        *bcd::bcd2num(self.market) as u8
    }
    pub const fn fcode(&self) -> u8 {
        *bcd::bcd2num(self.fcode) as u8
    }
    pub const fn fver(&self) -> u8 {
        *bcd::bcd2num(self.fver) as u8
    }
    pub fn seqno(&self) -> u64 {
        bcd::bcdarr2num(&self.seqno)
    }
}

#[derive(BytesCast, Debug, PartialEq)]
#[repr(C)]
pub struct TwExMdRtBase {
    // Tw Exchange Market data Realtime Base
    symbol: [u8; 6],
    pub time: bcd::TwExMdTimeu6,
    item_mask: u8,
    // - Bit 7 成交
    //   - 0︰無成交價、成交量，不傳送
    //   - 1︰有成交價、成交量，而且傳送
    // - Bit 6-4 買進
    //   - 000︰無買進價、買進量，不傳送
    //   - 001－101︰揭示買進價、買進量之傳送之檔位數（1..5檔）
    // - Bit 3-1 賣出
    //   - 000︰無賣出價、賣出量，不傳送
    //   - 001 －101︰揭示賣出價、賣出量之傳送之檔位數（1..5檔）
    // - Bit 0 最佳五檔價量 (Fmt23 保留)
    //   逐筆交易每筆委託撮合後，可能產生數個成交價量揭示，
    //   揭示最後一個成交價量時，同時揭露最佳五檔買賣價量，Bit 0 = 0。
    //   非最後一個成交價量揭示時，則僅揭示成交價量但不揭示最佳五檔，Bit 0 = 1。
    lmt_mask: u8,
    // 以各別 Bit 分別表示各項漲跌停註記、暫緩撮合瞬間價格趨勢及延後收盤註記（預設值均為 0x00）
    // - Bit 7-6 成交漲跌停註記 00：一般成交; 01：跌停成交; 10：漲停成交
    // - Bit 5-4 最佳一檔買進   00：一般買進; 01：跌停買進; 10：漲停買進
    // - Bit 3-2 最佳一檔賣出   00：一般賣出; 01：跌停賣出; 10：漲停賣出
    // - Bit 1-0 瞬間價格趨勢   00：一般揭示; 01：暫緩撮合且瞬間趨跌; 10：暫緩撮合且瞬間趨漲; 11：（保留）    
    status_mask: u8
    // - Bit 7 試算狀態註記
    //   - 若 Bit 7 為 1，表示目前即時行情:PQs_[] 為試算階段狀態；
    //   - 若 Bit 7 為 0，表示目前為一般揭示狀態，此時 Bit 6 與 Bit 5 註記資料無任何意義。
    // - Bit 6 試算後,延後開盤註記  0：否; 1：是     (Fmt23 保留)
    // - Bit 5 試算後,延後收盤註記  0：否; 1：是     (Fmt23 保留)
    // - Bit 4 撮合方式註記        0：集合競價; 1：逐筆撮合
    // - Bit 3 開盤註記            0：否; 1：是
    // - Bit 2 收盤註記            0：否; 1：是
    // - Bit 1-0 保留
}

impl TwExMdRtBase {
    pub fn symbol(&self) -> &str {
        //Result<&str, str::Utf8Error>{
        str::from_utf8(&self.symbol).unwrap()
    }
    pub const fn is_last_deal(&self) -> bool {
        return self.time.hraw() == 0x99
    }

    // pub const fn has_bs(&self) -> bool {
    //     (self.item_mask & 0x7e) || (self.item_mask & 1) == 0
    // }
    
    pub const fn n_match(&self) -> u8 {
        (self.item_mask & 0x80) >> 7
    }
    pub const fn n_bid(&self) -> u8 {
        (self.item_mask & 0x70) >>4
    }
    pub const fn n_ask(&self) -> u8 {
        (self.item_mask & 0x0e) >> 1
    }
    pub const fn trice(&self) -> u8 {
        self.lmt_mask & 0x03
    }
    pub const fn simulation(&self) -> bool {
        (self.status_mask & 0x80) != 0
    }
    pub const fn delay_open(&self) -> bool {
        (self.status_mask & 0x40) != 0
    }
    pub const fn delay_close(&self) -> bool {
        (self.status_mask & 0x20) != 0
    }
    pub const fn auction(&self) -> bool {
        (self.status_mask & 0x10) != 0
    }
    pub const fn is_open(&self) -> bool {
        (self.status_mask & 0x08) != 0
    }
    pub const fn is_close(&self) -> bool {
        (self.status_mask & 0x04) != 0
    }
    

}

#[cfg(test)]
extern crate test_case;

#[cfg(test)]
mod tests {
    use super::*;
    // use test_case::test_case;

    #[test]
    fn tw_ex_head_test() {
        let data = &[
            0x1b, 0x1, 0x31, 0x1, 0x6, 0x4, 0x0, 0x10, 0x93, 0x59, 0x39, 0x31, 0x31, 0x36, 0x31,
            0x36, 0x9, 0x0, 0x0, 0x14, 0x8, 0x66, 0xda, 0x0, 0x8, 0x0, 0x0, 0x0, 0x6, 0x0, 0x0,
            0x1, 0x82, 0x0, 0x0, 0x0, 0x0, 0x6, 0x0, 0x0, 0x1, 0x82, 0x0, 0x0, 0x0, 0x0, 0x6, 0x0,
            0x0, 0x1, 0x81, 0x0, 0x0, 0x0, 0x0, 0x5, 0x0, 0x0, 0x1, 0x80, 0x0, 0x0, 0x0, 0x0, 0x16,
            0x0, 0x0, 0x1, 0x76, 0x0, 0x0, 0x0, 0x0, 0x28, 0x0, 0x0, 0x1, 0x75, 0x0, 0x0, 0x0, 0x0,
            0x20, 0x0, 0x0, 0x1, 0x93, 0x0, 0x0, 0x0, 0x0, 0x8, 0x0, 0x0, 0x1, 0x94, 0x0, 0x0, 0x0,
            0x0, 0x1, 0x0, 0x0, 0x1, 0x95, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x1, 0x96, 0x0, 0x0,
            0x0, 0x0, 0x25, 0x0, 0x0, 0x1, 0x97, 0x0, 0x0, 0x0, 0x0, 0x26, 0xc6,
        ];
        let (head, _rest) = TWExMdHead::from_bytes(data).unwrap();
        let (base, _rest) = TwExMdRtBase::from_bytes(_rest).unwrap();
        assert_eq!(head.len(), 131);
        assert_eq!(head.market(), 1);
        assert_eq!(head.fcode(), 6);
        assert_eq!(head.fver(), 4);
        assert_eq!(head.seqno(), 109359);
        assert_eq!(base.symbol(), String::from("911616"));
        assert_eq!(base.time.to_string(), String::from("09:00:00.140866"));
    }
}
