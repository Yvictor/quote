use serde::{Deserialize, Deserializer};

fn deserialize_ascii<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let bytes: [u8; 1] = Deserialize::deserialize(deserializer)?;
    Ok(bytes[0])
    // String::from_utf8(bytes).map_err(serde::de::Error::custom)
}

fn deserialize_bcd_u32_len4<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let bytes: [u8; 4] = Deserialize::deserialize(deserializer)?;
    let result: u32 = ((bytes[0] >> 4) as u32 * 10000000)    // High 4 bits of byte 0 -> Ten million's place
                + ((bytes[0] & 0x0F) as u32 * 1000000)   // Low 4 bits of byte 0 -> Million's place
                + ((bytes[1] >> 4) as u32 * 100000)      // High 4 bits of byte 1 -> Hundred thousand's place
                + ((bytes[1] & 0x0F) as u32 * 10000)     // Low 4 bits of byte 1 -> Ten thousand's place
                + ((bytes[2] >> 4) as u32 * 1000)        // High 4 bits of byte 2 -> Thousand's place
                + ((bytes[2] & 0x0F) as u32 * 100)       // Low 4 bits of byte 2 -> Hundred's place
                + ((bytes[3] >> 4) as u32 * 10)          // High 4 bits of byte 3 -> Ten's place
                + (bytes[3] & 0x0F) as u32;              // Low 4 bits of byte 3 -> One's place
    Ok(result)
}

fn deserialize_bcd_u16<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    let bytes: [u8; 2] = Deserialize::deserialize(deserializer)?;
    let result: u16 = ((bytes[0] >> 4) as u16 * 1000) + ((bytes[0] & 0x0F) as u16 * 100) + ((bytes[1] >> 4) as u16 * 10) + (bytes[1] & 0x0F) as u16;
    // let mut result = 0;
    // for byte in bytes {
    //     result = result * 100 + ((byte >> 4) as u16 * 10) + (byte & 0x0F) as u16;
    // }
    Ok(result)
}

fn deserialize_bcd_u8<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let bytes: [u8; 1] = Deserialize::deserialize(deserializer)?;
    let result: u8 = ((bytes[0] >> 4) as u8 * 10) + (bytes[0] & 0x0F) as u8;
    Ok(result)
}

// fn deserialize_bcd_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let bytes: Vec<u8> = Deserialize::deserialize(deserializer)?;
//     let mut result = 0;
//     for byte in bytes {
//         result = result * 100 + ((byte >> 4) as u32 * 10) + (byte & 0x0F) as u32;
//     }
//     Ok(result)
// }


macro_rules! generate_deserializer {
    // 針對 ASCII 解析器
    ($name:ident, $len:expr, $output:ty, ascii) => {
        fn $name<'de, D>(deserializer: D) -> Result<$output, D::Error>
        where
            D: Deserializer<'de>,
        {
            let bytes: [u8; $len] = Deserialize::deserialize(deserializer)?;
            Ok(bytes[0] as $output) // 對於 ASCII，我們假設是單字節數據
        }
    };

    // 針對 Pack BCD 解析器
    ($name:ident, $len:expr, $output:ty, packbcd) => {
        fn $name<'de, D>(deserializer: D) -> Result<$output, D::Error>
        where
            D: Deserializer<'de>,
        {
            let bytes: [u8; $len] = Deserialize::deserialize(deserializer)?;
            let mut result: $output = 0;
            for byte in bytes.iter() {
                result = result * 100 + ((<$output>::from(byte >> 4)) * 10) + (<$output>::from(byte & 0x0F));
            }
            Ok(result)
        }
    };
}

generate_deserializer!(de_ascii_u8, 1, u8, ascii);
generate_deserializer!(de_bcd_u16, 2, u16, packbcd);
generate_deserializer!(de_bcd_u8, 1, u8, packbcd);
generate_deserializer!(de_bcd_u32_len4, 4, u32, packbcd);

// generate_deserializer!(deserialize_ascii_u16, 2, u16, ascii);

#[derive(Debug, Deserialize)]
pub struct Header {
    #[serde(deserialize_with = "de_ascii_u8")]  // ESC-CODE X(01)
    pub esc_code: u8,

    #[serde(deserialize_with = "de_bcd_u16")]  // 訊息長度 9(04)
    pub message_length: u16,

    #[serde(deserialize_with = "de_bcd_u8")]  // 業務別 9(02)
    pub business_code: u8,

    #[serde(deserialize_with = "de_bcd_u8")]  // 傳輸格式代碼 9(02)
    pub format_code: u8,

    #[serde(deserialize_with = "de_bcd_u8")]  // 傳輸格式版別 9(02)
    pub format_version: u8,

    #[serde(deserialize_with = "de_bcd_u32_len4")]  // 傳輸序號 9(08)
    pub transmission_number: u32,
}

fn deserialize_ascii_str6<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let bytes: [u8; 6] = Deserialize::deserialize(deserializer)?;
    let str = std::str::from_utf8(&bytes).map_err(serde::de::Error::custom)?;
    Ok(str.to_string())
}

fn parse_bcd_u8(byte: u8) -> u8 {
    ((byte >> 4) * 10) + (byte & 0x0F)
}

fn parse_bcd_u16(high_byte: u8, low_byte: u8) -> u16 {
    (parse_bcd_u8(high_byte) as u16 * 100) + parse_bcd_u8(low_byte) as u16
}

fn deserialize_bcd_time<'de, D>(deserializer: D) -> Result<Time, D::Error>
where
    D: Deserializer<'de>,
{
    let bytes: [u8; 6] = Deserialize::deserialize(deserializer)?;

    let hours = parse_bcd_u8(bytes[0]);
    let minutes = parse_bcd_u8(bytes[1]);
    let seconds = parse_bcd_u8(bytes[2]);
    let milliseconds = ((parse_bcd_u8(bytes[3]) as u16) * 10) + (bytes[4] >> 4) as u16;
    let microseconds = parse_bcd_u16(bytes[4] & 0x0F, bytes[5]);

    Ok(Time(hours, minutes, seconds, milliseconds, microseconds))
}


#[derive(Debug, PartialEq)]
pub struct Time(u8, u8, u8, u16, u16);


#[derive(Debug, PartialEq)]
pub struct RevealItems {
    pub has_trade: bool,          // Bit 7
    pub buy_price_levels: u8,     // Bit 6-4 (1-5 檔)
    pub sell_price_levels: u8,    // Bit 3-1 (1-5 檔)
    pub only_trade: bool,         // Bit 0
}

fn deserialize_bitmap<'de, D>(deserializer: D) -> Result<RevealItems, D::Error>
where
    D: Deserializer<'de>,
{
    // 我們從序列化的數據中讀取 1 個字節
    let byte: u8 = Deserialize::deserialize(deserializer)?;

    // 根據 bit 位置解析各個欄位
    let has_trade = ((byte & 0b10000000) >> 7) == 1;      // Bit 7: 是否有成交價、成交量
    let buy_price_levels = (byte & 0b01110000) >> 4;     // Bit 6-4: 買進價量檔數
    let sell_price_levels = (byte & 0b00001110) >> 1;    // Bit 3-1: 賣出價量檔數
    let only_trade = (byte & 0b00000001) != 0;      // Bit 0: 是否僅揭示成交價量

    // 將解析後的值包裝成 RevealItems 結構體
    Ok(RevealItems {
        has_trade,
        buy_price_levels,
        sell_price_levels,
        only_trade,
    })
}


#[derive(Debug, PartialEq)]
pub enum LimitType {
    Normal, // 00 一般成交 00：一般買進 00：一般賣出
    LimitUp, // 01 跌停成交 01：跌停買進 10：跌停賣出
    LimitDown, // 10 漲停成交 10：漲停買進 10：漲停賣出
}

impl LimitType {
    fn from_bits(bits: u8) -> Self {
        match bits {
            0b00 => LimitType::Normal,
            0b01 => LimitType::LimitUp,
            0b10 => LimitType::LimitDown,
            _ => LimitType::Normal, // 假設不會出現其他值
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum PriceTrend {
    Normal,
    Down, // 01 暫緩撮合且瞬間趨跌
    Up,   // 10 暫緩撮合且瞬間趨漲
}

impl PriceTrend {
    fn from_bits(bits: u8) -> Self {
        match bits {
            0b00 => PriceTrend::Normal,
            0b01 => PriceTrend::Down,
            0b10 => PriceTrend::Up,
            _ => PriceTrend::Normal, // 假設不會出現其他值
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct LimitNote {
    pub trade: LimitType,       // Bit 7-6: 成交漲跌停註記 00：一般成交 	01：跌停成交 	10：漲停成交 	
    pub buy: LimitType,         // Bit 5-4: 最佳一檔買進漲跌停註記 00：一般買進 	01：跌停買進 	10：漲停買進 	
    pub sell: LimitType,        // Bit 3-2: 最佳一檔賣出漲跌停註記 00：一般賣出 	01：跌停賣出 	10：漲停賣出
    pub price_trend: PriceTrend,       // Bit 1-0: 瞬間價格趨勢 00：一般揭示  01：暫緩撮合且瞬間趨跌  10：暫緩撮合且瞬間趨漲  11：（保留）
}

fn deserialize_bitmap_limit_note<'de, D>(deserializer: D) -> Result<LimitNote, D::Error>
where
    D: Deserializer<'de>,
{
    // 我們從序列化的數據中讀取 1 個字節
    let byte: u8 = Deserialize::deserialize(deserializer)?;

    // 根據位元位置來解析各個欄位
    let trade = LimitType::from_bits((byte & 0b11000000) >> 6); // Bit 7-6
    let buy = LimitType::from_bits((byte & 0b00110000) >> 4);   // Bit 5-4
    let sell = LimitType::from_bits((byte & 0b00001100) >> 2);  // Bit 3-2
    let price_trend = PriceTrend::from_bits(byte & 0b00000011); // Bit 1-0

    // 返回解析後的結構體
    Ok(LimitNote {
        trade,
        buy,
        sell,
        price_trend,
    })
}

#[derive(Debug, PartialEq)]
pub enum MatchingMethod {
    Auction,  // 集合競價
    Continuous,  // 逐筆撮合
}

impl MatchingMethod {
    fn from_bits(bits: u8) -> Self {
        match bits {
            0b00 => MatchingMethod::Auction,
            0b01 => MatchingMethod::Continuous,
            _ => MatchingMethod::Auction,
        }
    }
}
#[derive(Debug, PartialEq)]
pub struct StatusNote {
    pub simulation_status: bool,    // Bit 7: 試算狀態註記
    pub delayed_opening: bool,      // Bit 6: 試算後延後開盤註記
    pub delayed_closing: bool,      // Bit 5: 試算後延後收盤註記
    pub matching_method: MatchingMethod,      // Bit 4: 撮合方式註記
    pub opening_status: bool,       // Bit 3: 開盤註記
    pub closing_status: bool,       // Bit 2: 收盤註記
}

impl<'de> Deserialize<'de> for StatusNote {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let byte: u8 = Deserialize::deserialize(deserializer)?;

        // 根據每個位元的值解析為布林
        let simulation_status = (byte & 0b10000000) != 0; // Bit 7: 試算狀態註記
        let delayed_opening = (byte & 0b01000000) != 0;   // Bit 6: 試算後延後開盤註記
        let delayed_closing = (byte & 0b00100000) != 0;   // Bit 5: 試算後延後收盤註記
        let matching_method = MatchingMethod::from_bits((byte & 0b00010000) >> 4);   // Bit 4: 撮合方式註記
        let opening_status = (byte & 0b00001000) != 0;    // Bit 3: 開盤註記
        let closing_status = (byte & 0b00000100) != 0;    // Bit 2: 收盤註記

        // 返回解析後的結構體
        Ok(StatusNote {
            simulation_status,
            delayed_opening,
            delayed_closing,
            matching_method,
            opening_status,
            closing_status,
        })
    }
}


#[derive(Debug, Deserialize)]
pub struct Body {
    #[serde(deserialize_with = "deserialize_ascii_str6")]  // 股票代號 X(06)
    pub stock_code: String,

    #[serde(deserialize_with = "deserialize_bcd_time")]  // 撮合時間 9(12)
    pub match_time: Time,

    #[serde(deserialize_with = "deserialize_bitmap")]  // 揭示項目註記 X(01)
    pub reveal_items: RevealItems,

    #[serde(deserialize_with = "deserialize_bitmap_limit_note")]  // 漲跌停註記 X(01)
    pub limit_note: LimitNote,

    // #[serde(deserialize_with = "deserialize_bitmap")]  // 狀態註記 X(01)
    pub status_note: StatusNote,

    #[serde(deserialize_with = "de_bcd_u32_len4")]  // 累計成交數量 9(08)
    pub cumulative_volume: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bincode;

    #[test]
    fn test_header_parsing() {
        // 模擬的 HEADER 二進制數據
        let data = vec![
            0x1B,                   // ESC-CODE
            0x01, 0x13,             // 訊息長度 113 (BCD)
            0x01,                   // 業務別 "01" (BCD)
            0x06,                   // 傳輸格式代碼 "06" (BCD)
            0x04,                   // 傳輸格式版別 "04" (BCD)
            0x00, 0x00, 0x45, 0x67, // 傳輸序號 4567 (BCD)
        ];

        // 反序列化為 Header 結構體
        let header: Header = bincode::deserialize(&data).unwrap();

        // 檢查解析出的欄位是否正確
        assert_eq!(header.esc_code, 0x1B);      // ESC-CODE 應該是 ASCII 27 (0x1B)
        assert_eq!(header.message_length, 113);   // 訊息長度應該是 113
        assert_eq!(header.business_code, 1);      // 業務別應該是 1
        assert_eq!(header.format_code, 6);        // 傳輸格式代碼應該是 6
        assert_eq!(header.format_version, 4);     // 傳輸格式版別應該是 4
        assert_eq!(header.transmission_number, 4567); // 傳輸序號應該是 4567
    }


    #[test]
    fn test_body_parsing() {
        // 模擬的 BODY 二進制數據
        let data = vec![
            0x32, 0x33, 0x33, 0x30, 0x20, 0x20, // 股票代號 "2330  " (ASCII)
            0x09, 0x04, 0x15, 0x06, 0x12, 0x78, // 撮合時間 09:04:15.061278 (BCD)
            0xD6, // 揭示項目：7 1 ：有成交價、成交量 
                            // 6-4 101：有最佳五檔買進價
                            // 3-1 011：有最佳三檔賣出價
                            // 0 0 ：僅揭示成交價量
            0x00, // 漲跌停註記 一般成交、買進、賣出且為一般揭示
            0x00, // 狀態註記 試算狀態註記、試算後延後開盤註記、試算後延後收盤註記、集合競價、開盤、收盤
            0x00, 0x01, 0x64, 0x23, // 累計成交數量 16423
        ];

        // 反序列化為 Body 結構體
        let body: Body = bincode::deserialize(&data).unwrap();

        // 檢查解析出的欄位是否正確
        assert_eq!(body.stock_code, "2330  "); // 股票代號應該是 "2330  "
        assert_eq!(body.match_time, Time(9, 4, 15, 061, 278)); // 撮合時間應該是 09:04:15.061278
        assert_eq!(body.reveal_items, RevealItems {
            has_trade: true,
            buy_price_levels: 5,
            sell_price_levels: 3,
            only_trade: false,
        });
        assert_eq!(body.limit_note, LimitNote {
            trade: LimitType::Normal,
            buy: LimitType::Normal,
            sell: LimitType::Normal,
            price_trend: PriceTrend::Normal,
        });
        assert_eq!(body.status_note, StatusNote {
            simulation_status: false,
            delayed_opening: false,
            delayed_closing: false,
            matching_method: MatchingMethod::Auction,
            opening_status: false,
            closing_status: false,
        });
        assert_eq!(body.cumulative_volume, 16423);
    }
}

// use nom::bytes::complete::{take, take_while_m_n};
// use nom::number::complete::{be_u8, be_u16, be_u32};
// use nom::IResult;
// use std::str;


// #[derive(Debug, PartialEq)]
// pub enum BCDValue {
//     U8(u8),
//     U16(u16),
//     U32(u32),
//     U64(u64),
// }


// #[derive(Debug, PartialEq)]
// pub struct MarketData {
//     esc_code: u8,
//     message_length: u16,
//     business_type: u8,
//     format_code: u8,
//     format_version: u8,
//     transmission_seq: u32,
//     stock_code: String,
//     match_time: String,
//     cumulative_quantity: u32,
//     price: Vec<f64>,
//     quantity: Vec<u32>,
//     checksum: u8,
// }

// fn parse_bcd(input: &[u8], length: usize) -> IResult<&[u8], String> {
//     let (input, bytes) = take(length)(input)?;
//     let decoded: String = bytes
//         .iter()
//         .map(|byte| format!("{:02X}", byte))
//         .collect();
//     Ok((input, decoded))
// }

// fn parse_price(input: &[u8]) -> IResult<&[u8], f64> {
//     let (input, price_bytes) = take(5usize)(input)?;
//     let price: u64 = price_bytes.iter().fold(0u64, |acc, &b| acc * 100 + b as u64);
//     Ok((input, price as f64 / 10000.0))
// }

// fn parse_quantity(input: &[u8]) -> IResult<&[u8], u32> {
//     let (input, quantity_bytes) = take(4usize)(input)?;
//     let quantity: u32 = quantity_bytes.iter().fold(0u32, |acc, &b| acc * 100 + b as u32);
//     Ok((input, quantity))
// }

// fn parse_market_data(input: &[u8]) -> IResult<&[u8], MarketData> {
//     let (input, esc_code) = be_u8(input)?;
//     let (input, message_length) = be_u16(input)?;

//     let (input, business_type) = be_u8(input)?;
//     let (input, format_code) = be_u8(input)?;
//     let (input, format_version) = be_u8(input)?;
//     let (input, transmission_seq) = be_u32(input)?;

//     let (input, stock_code) = take(6usize)(input)?;
//     let stock_code = str::from_utf8(stock_code).unwrap().trim().to_string();

//     let (input, match_time) = parse_bcd(input, 6)?;

//     let (input, _reveal_item_flag) = be_u8(input)?;
//     let (input, _price_limit_flag) = be_u8(input)?;
//     let (input, _status_flag) = be_u8(input)?;

//     let (input, cumulative_quantity) = parse_quantity(input)?;

//     // 可变长度的价格和数量
//     let mut price = Vec::new();
//     let mut quantity = Vec::new();
//     let mut rest = input;

//     while rest.len() > 5 {
//         let (new_rest, p) = parse_price(rest)?;
//         let (new_rest, q) = parse_quantity(new_rest)?;
//         price.push(p);
//         quantity.push(q);
//         rest = new_rest;
//     }

//     let (rest, checksum) = be_u8(rest)?;

//     Ok((
//         rest,
//         MarketData {
//             esc_code,
//             message_length,
//             business_type,
//             format_code,
//             format_version,
//             transmission_seq,
//             stock_code,
//             match_time,
//             cumulative_quantity,
//             price,
//             quantity,
//             checksum,
//         },
//     ))
// }

// pub fn parse_bcd_u16(input: &[u8]) -> IResult<&[u8], u16> {
//     let (input, bytes) = take(2usize)(input)?;
//     let value = ((bytes[0] >> 4) as u16) * 1000        // 第1个字节的高4位表示千位
//               + ((bytes[0] & 0x0F) as u16) * 100       // 第1个字节的低4位表示百位
//               + ((bytes[1] >> 4) as u16) * 10          // 第2个字节的高4位表示十位
//               + (bytes[1] & 0x0F) as u16;              // 第2个字节的低4位表示个位
//     Ok((input, value))
// }

// pub fn parse_bcd_u8(input: &[u8]) -> IResult<&[u8], u8> {
//     let (input, byte) = take(1usize)(input)?;
//     let value = ((byte[0] >> 4) * 10) + (byte[0] & 0x0F); // 高 4 位乘以 10，加上低 4 位
//     // let value = (((byte[0] >> 4) & 0x0F) * 10) + (byte[0] & 0x0F); // 高 4 位乘以 10，加上低 4 位
//     Ok((input, value))
// }


// // pub fn parse_bcd_u64(input: &[u8], min_len: usize, max_len: usize) -> IResult<&[u8], u64> {
// //     let (input, bytes) = take_while_m_n(min_len, max_len, is_bcd_digit)(input)?;

// //     let mut value: u64 = 0;
// //     for &byte in bytes {
// //         value = value * 100 + ((byte >> 4) as u64) * 10 + (byte & 0x0F) as u64;
// //     }

// //     Ok((input, value))
// // }


// pub fn parse_bcd_value(input: &[u8], length: usize) -> IResult<&[u8], BCDValue> {
//     match length {
//         1 => {
//             let (input, byte) = take(1usize)(input)?;
//             let value = ((byte[0] >> 4) * 10) + (byte[0] & 0x0F);
//             Ok((input, BCDValue::U8(value)))
//         }
//         2 => {
//             let (input, bytes) = take(2usize)(input)?;
//             let value: u16 = ((bytes[0] >> 4) as u16 * 1000) + ((bytes[0] & 0x0F) as u16 * 100) + ((bytes[1] >> 4) as u16 * 10) + (bytes[1] & 0x0F) as u16;
//             Ok((input, BCDValue::U16(value)))
//         }
//         3 => {
//             let (input, bytes) = take(3usize)(input)?;
//             let value: u32 = ((bytes[0] >> 4) as u32 * 100000)    // 高4位 -> 万位
//                            + ((bytes[0] & 0x0F) as u32 * 10000)   // 低4位 -> 千位
//                            + ((bytes[1] >> 4) as u32 * 1000)      // 高4位 -> 百位
//                            + ((bytes[1] & 0x0F) as u32 * 100)     // 低4位 -> 十位
//                            + ((bytes[2] >> 4) as u32 * 10)        // 高4位 -> 个位
//                            + (bytes[2] & 0x0F) as u32;            // 低4位 -> 小数
            
//             Ok((input, BCDValue::U32(value)))
//         }
//         4 => {
//             let (input, bytes) = take(4usize)(input)?;
            
//             // Parse 4 bytes into a 32-bit integer
//             let value: u32 = ((bytes[0] >> 4) as u32 * 10000000)    // High 4 bits of byte 0 -> Ten million's place
//                            + ((bytes[0] & 0x0F) as u32 * 1000000)   // Low 4 bits of byte 0 -> Million's place
//                            + ((bytes[1] >> 4) as u32 * 100000)      // High 4 bits of byte 1 -> Hundred thousand's place
//                            + ((bytes[1] & 0x0F) as u32 * 10000)     // Low 4 bits of byte 1 -> Ten thousand's place
//                            + ((bytes[2] >> 4) as u32 * 1000)        // High 4 bits of byte 2 -> Thousand's place
//                            + ((bytes[2] & 0x0F) as u32 * 100)       // Low 4 bits of byte 2 -> Hundred's place
//                            + ((bytes[3] >> 4) as u32 * 10)          // High 4 bits of byte 3 -> Ten's place
//                            + (bytes[3] & 0x0F) as u32;              // Low 4 bits of byte 3 -> One's place
            
//             Ok((input, BCDValue::U32(value)))
//         }
//         5 => {
//             let (input, bytes) = take(5usize)(input)?;
            
//             // Parse 5 bytes into a 64-bit integer
//             let value: u64 = ((bytes[0] >> 4) as u64 * 1000000000)   // High 4 bits of byte 0 -> Billion's place
//                            + ((bytes[0] & 0x0F) as u64 * 100000000)  // Low 4 bits of byte 0 -> Hundred million's place
//                            + ((bytes[1] >> 4) as u64 * 10000000)     // High 4 bits of byte 1 -> Ten million's place
//                            + ((bytes[1] & 0x0F) as u64 * 1000000)    // Low 4 bits of byte 1 -> Million's place
//                            + ((bytes[2] >> 4) as u64 * 100000)       // High 4 bits of byte 2 -> Hundred thousand's place
//                            + ((bytes[2] & 0x0F) as u64 * 10000)      // Low 4 bits of byte 2 -> Ten thousand's place
//                            + ((bytes[3] >> 4) as u64 * 1000)         // High 4 bits of byte 3 -> Thousand's place
//                            + ((bytes[3] & 0x0F) as u64 * 100)        // Low 4 bits of byte 3 -> Hundred's place
//                            + ((bytes[4] >> 4) as u64 * 10)           // High 4 bits of byte 4 -> Ten's place
//                            + (bytes[4] & 0x0F) as u64;               // Low 4 bits of byte 4 -> One's place
            
//             Ok((input, BCDValue::U64(value)))
//         }
//         _ => {
//             let (input, bytes) = take(length)(input)?;
//             let mut value: u64 = 0;
//             for &byte in bytes {
//                 value = value * 100 + ((byte >> 4) as u64) * 10 + (byte & 0x0F) as u64;
//             }
//             Ok((input, BCDValue::U64(value)))
//         } 
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_parse_bcd_u64() {
//         let data: [u8; 2] = [0x01, 0x13];
//         let (_, value) = parse_bcd_value(&data, 2).unwrap();
//         assert_eq!(value, BCDValue::U16(113));
//     }

//     #[test]
//     fn test_parse_bcd_u64_3() {
//         let data: [u8; 3] = [0x99, 0x13, 0x01];
//         let (_, value) = parse_bcd_value(&data, 3).unwrap();
//         assert_eq!(value, BCDValue::U32(991301));
//     }

//     #[test]
//     fn test_parse_bcd_u64_4() {
//         let data: [u8; 4] = [0x99, 0x99, 0x05, 0x06];
//         let (_, value) = parse_bcd_value(&data, 4).unwrap();
//         assert_eq!(value, BCDValue::U32(99990506));
//     }

//     #[test]
//     fn test_parse_bcd_u64_5() {
//         let data: [u8; 5] = [0x99, 0x13, 0x01, 0x05, 0x06];
//         let (_, value) = parse_bcd_value(&data, 5).unwrap();
//         assert_eq!(value, BCDValue::U64(9913010506));
//     }

//     #[test]
//     fn test_parse_format_code() {
//         // 假设传输格式代碼为 06（0x06 对应于 PACK BCD 编码的 06）
//         let data: [u8; 1] = [0x87];

//         // 调用解析函数
//         let (_, format_code) = parse_bcd_value(&data, 1).unwrap();

//         // 检查是否解析正确
//         assert_eq!(format_code, BCDValue::U8(87));
//     }
//     #[test]
//     fn test_parse_message_length() {
//         // 假设訊息長度為113（0x0113 对应于 PACK BCD 编码的 0113）
//         let data: [u8; 2] = [0x01, 0x13];

//         // 调用解析函数
//         let (_, message_length) = parse_bcd_u16(&data).unwrap();

//         // 检查是否解析正确
//         assert_eq!(message_length, 113);
//     }

//     #[test]
//     fn test_parse_market_data() {
//         let data: [u8; 113] = [
//             0x1B,                         // ESC-CODE: ASCII 27 (0x1B)
//             0x01, 0x13,                   // 訊息長度: 113 (0x0113)
//             0x01,                         // 業務別: 01 (集中市場普通股競價交易)
//             0x06,                         // 傳輸格式代碼: 06 (格式六)
//             0x04,                         // 傳輸格式版別: 04 (第4版)
//             0x00, 0x00, 0x45, 0x67,       // 傳輸序號: 4567 (0x00004567)
//             0x32, 0x33, 0x33, 0x30,       // 股票代號: "2330" (ASCII: 0x32 0x33 0x33 0x30)
//             0x20, 0x20,                   // 股票代號后补空格 (ASCII: 0x20 0x20)
//             0x09, 0x04, 0x15, 0x06,       // 撮合時間: 9 时 4 分 15 秒 61 毫秒 (0x09041506)
//             0x12, 0x78,                   // 撮合時間: 278 微秒 (0x1278)
//             0xD6,                         // 揭示項目註記: 0xD6 (揭示項目)
//             0x00,                         // 漲跌停註記: 0x00 (一般状态)
//             0x00,                         // 狀態註記: 0x00 (一般揭示，採集合競價)
//             0x00, 0x01, 0x64, 0x23,       // 累計成交量: 16423 (0x00016423)
//             0x00, 0x00, 0x99, 0x50, 0x00, // 成交價: 99.5000 (0x0000995000)
//             0x00, 0x00, 0x12, 0x34,       // 成交量: 1234 (0x00001234)
//             0x00, 0x00, 0x99, 0x50, 0x00, // 買進價1: 99.5000 (0x0000995000)
//             0x00, 0x00, 0x02, 0x50,       // 買進量1: 250 (0x00000250)
//             0x00, 0x00, 0x99, 0x00, 0x00, // 買進價2: 99.0000 (0x0000990000)
//             0x00, 0x00, 0x01, 0x75,       // 買進量2: 175 (0x00000175)
//             0x00, 0x00, 0x98, 0x50, 0x00, // 買進價3: 98.5000 (0x0000985000)
//             0x00, 0x00, 0x04, 0x77,       // 買進量3: 477 (0x00000477)
//             0x00, 0x00, 0x97, 0x50, 0x00, // 買進價4: 97.5000 (0x0000975000)
//             0x00, 0x00, 0x06, 0x69,       // 買進量4: 669 (0x00000669)
//             0x00, 0x00, 0x97, 0x00, 0x00, // 買進價5: 97.0000 (0x0000970000)
//             0x00, 0x00, 0x01, 0x25,       // 買進量5: 125 (0x00000125)
//             0x00, 0x01, 0x00, 0x00, 0x00, // 賣出價1: 100.0000 (0x0001000000)
//             0x00, 0x00, 0x00, 0x80,       // 賣出量1: 80 (0x00000080)
//             0x00, 0x01, 0x00, 0x50, 0x00, // 賣出價2: 100.5000 (0x0001005000)
//             0x00, 0x00, 0x06, 0x75,       // 賣出量2: 675 (0x00000675)
//             0x00, 0x01, 0x01, 0x50, 0x00, // 賣出價3: 101.5000 (0x0001015000)
//             0x00, 0x00, 0x04, 0x60,       // 賣出量3: 460 (0x00000460)
//             0xFC,                         // 檢查碼: 0xFC (2~110 Bytes XOR 值)
//             0x0D, 0x0A                    // TERMINAL-CODE: 0x0D 0x0A (固定值 HEXACODE)
//         ];

//         let expected = MarketData {
//             esc_code: 0x1B,
//             message_length: 113,
//             business_type: 1,
//             format_code: 6,
//             format_version: 4,
//             transmission_seq: 4567,
//             stock_code: "2330".to_string(),
//             match_time: "090415061278".to_string(),
//             cumulative_quantity: 16423,
//             price: vec![99.50, 99.5000, 99.0000, 98.5000, 97.5000, 97.0000, 100.0000, 100.5000, 101.5000],
//             quantity: vec![1234, 250, 175, 477, 669, 125, 80, 675, 460],
//             checksum: 0xFC,
//         };

//         let (_, result) = parse_market_data(&data).unwrap();
//         assert_eq!(result, expected);
//     }
// }