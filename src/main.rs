use quote::add_two;

fn main() {
    println!("{}", format!("Hello, Rust! {value}", value=add_two(1)));
}
