use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use crate::paser::f6::{bytes2quote, bytes2header, bytes2mlen, F6};


pub fn readf6file(path: &Path, rec_handler: fn(F6)){
    let display = path.display();
    let mut buf = [0u8; 256];
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => (file)
    };
    let file_size = file.metadata().unwrap().len();
    loop {
        if file.stream_position().unwrap() == file_size {
            break
        }
        match file.read_exact(&mut buf[..4]) {
            Ok(_) => {
                let mlen = bytes2mlen(&buf);
                file.read_exact(&mut buf[4..mlen]).unwrap();
                //print!("{} contains:\n{:?}", display, &header[..]);
                let h = bytes2header(&buf);
                // println!("header: {:?}", h);
                let (n_match, n_bid, n_ask) = h.n_info();
                let f6 = F6 {
                    header: h,
                    quote: bytes2quote(&buf[29..mlen], n_match, n_bid, n_ask),
                };
                rec_handler(f6);
            }
            Err(why) => {
                println!("couldn't read {}: {}", display, why);
                break;
            },
        }
    }
    

}

#[cfg(test)]
mod tests {
    use super::*;
    // use test_case::test_case;

    #[test]
    fn read_test() {
        fn f6handler (f6: F6) {
            println!("{:?}", f6);
        }
        readf6file(&Path::new("tests/data/f6_01000001_01001000_TP03.new"), f6handler);
        assert_eq!(1, 1)
    }
}