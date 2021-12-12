use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use crate::paser::f6::{bytes2quote, bytes2header, F6};


pub fn readf6file(path: &Path, rec_handler: fn(F6)){
    let display = path.display();
    let mut header = [0u8; 29];
    let mut body = [0u8; 256];
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => (file)
    };
    let file_size = file.metadata().unwrap().len();
    loop {
        if file.stream_position().unwrap() == file_size {
            break
        }
        match file.read_exact(&mut header) {
            Ok(_) => {
                //print!("{} contains:\n{:?}", display, &header[..]);
                let h = bytes2header(&header);
                // println!("header: {:?}", h);
                let (n_match, n_bid, n_ask) = h.n_info();
                let body_size = (h.mlen - 29) as usize;
                // println!("bodsize: {:?}", body_size);
                file.read_exact(&mut body[0..body_size]).unwrap();
                let f6 = F6 {
                    header: h,
                    quote: bytes2quote(&body, n_match, n_bid, n_ask),
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
    use test_case::test_case;

    #[test]
    fn read_test() {
        fn f6handler (f6: F6) {
            println!("{:?}", f6);
        }
        readf6file(&Path::new("tests/data/f6_01000001_01001000_TP03.new"), f6handler);
        assert_eq!(1, 1)
    }
}