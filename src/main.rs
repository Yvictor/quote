use quote::paser::bcd::bcd2str;

fn main() {
    println!("{}", format!("Hello, Rust! {value}", value=bcd2str(128)));
}
