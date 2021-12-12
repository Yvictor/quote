use std::fs::File;
use std::io::BufReader;
use std::path::Path;
// use std::io::prelude::*;
use crate::paser::f6::{bytes2header, bytes2mlen, bytes2quote, bytes2f6, F6};
use filebuffer::FileBuffer;
use std::io::{Cursor, Read, Seek, SeekFrom};
use rayon::prelude::*;

pub fn readf6file(path: &Path, rec_handler: fn(F6)) {
    let display = path.display();
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => (file),
    };
    let file_size = file.metadata().unwrap().len();
    let mut fbuffer = vec![0u8; file_size as usize];
    file.read(&mut fbuffer).unwrap();
    let mut buf = [0u8; 256];
    let mut c = Cursor::new(fbuffer);
    c.seek(SeekFrom::Start(0)).unwrap();
    loop {
        if c.position() == file_size {
            break;
        }
        c.read_exact(&mut buf[..4]).unwrap();
        let mlen = bytes2mlen(&buf);
        c.read_exact(&mut buf[4..mlen]).unwrap();
        let h = bytes2header(&buf);
        // println!("header: {:?}", h);
        let (n_match, n_bid, n_ask) = h.n_info();
        let f6 = F6 {
            header: h,
            quote: bytes2quote(&buf[29..mlen], n_match, n_bid, n_ask),
        };
        rec_handler(f6);
    }
}

pub fn readf6bufreader(path: &Path, rec_handler: fn(F6)) {
    let display = path.display();
    let file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => (file),
    };
    let file_size = file.metadata().unwrap().len();
    let mut reader = BufReader::new(file);
    let mut buf = [0u8; 256];
    // c.seek(SeekFrom::Start(0)).unwrap();
    loop {
        if reader.stream_position().unwrap() == file_size {
            break;
        }
        reader.read_exact(&mut buf[..4]).unwrap();
        let mlen = bytes2mlen(&buf);
        reader.read_exact(&mut buf[4..mlen]).unwrap();
        let h = bytes2header(&buf);
        // println!("header: {:?}", h);
        let (n_match, n_bid, n_ask) = h.n_info();
        let f6 = F6 {
            header: h,
            quote: bytes2quote(&buf[29..mlen], n_match, n_bid, n_ask),
        };
        rec_handler(f6);
    }
}

pub fn readf6filebuffer(path: &Path, rec_handler: fn(F6)) {
    // let display = path.display();
    let fbuffer = FileBuffer::open(&path).expect("failed to open file {}");
    let fsize: u64 = fbuffer.len() as u64;
    // let mut buf = [0u8; 256];
    // let mut bufarr = vec![[0u8; 256]; ((fsize / 131) + 1) as usize];
    const BUFSIZE: usize = 2048;
    let mut bufarr = [[0u8; 256]; BUFSIZE];
    let mut c = Cursor::new(fbuffer);
    c.seek(SeekFrom::Start(0)).unwrap();
    let mut count = 0;
    loop {
        if c.position() == fsize {
            bufarr[..count].into_par_iter().for_each(|x| {
                let f6 = bytes2f6(x);
                rec_handler(f6);
            });
            break;
        }
        c.read_exact(&mut bufarr[count][..4]).unwrap();
        let mlen = bytes2mlen(&bufarr[count]);
        c.read_exact(&mut bufarr[count][4..mlen]).unwrap();
        count += 1;
        if count == BUFSIZE {
            bufarr.into_par_iter().for_each(|x| {
                let f6 = bytes2f6(&x);
                rec_handler(f6);
            });
            count = 0;
            // bufarr.push([0u8; 256]);
        }
        // let h = bytes2header(&buf);
        // // println!("header: {:?}", h);
        // let (n_match, n_bid, n_ask) = h.n_info();
        // let f6 = F6 {
        //     header: h,
        //     quote: bytes2quote(&buf[29..mlen], n_match, n_bid, n_ask),
        // };
        // rec_handler(f6);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use test_case::test_case;

    #[test]
    fn readf6file_test() {
        fn f6handler(f6: F6) {
            println!("{:?}", f6);
        }
        readf6file(
            &Path::new("tests/data/f6_01000001_01001000_TP03.new"),
            f6handler,
        );
        assert_eq!(1, 1)
    }

    #[test]
    fn readf6bufreader_test() {
        fn f6handler(f6: F6) {
            println!("{:?}", f6);
        }
        readf6bufreader(
            &Path::new("tests/data/f6_01000001_01001000_TP03.new"),
            f6handler,
        );
        assert_eq!(1, 1)
    }

    #[test]
    fn readf6filebuffer_test() {
        fn f6handler(f6: F6) {
            println!("{:?}", f6);
        }
        readf6filebuffer(
            &Path::new("tests/data/f6_01000001_01001000_TP03.new"),
            f6handler,
        );
        assert_eq!(1, 1)
    }
}
