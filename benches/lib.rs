extern crate quote;
 
#[macro_use]
extern crate bencher;
 
use quote::*;
 
use bencher::Bencher;
 
fn benchmark_bcd2str(bencher: &mut Bencher) {
    bencher.iter(|| bcd2str(128));
}

fn benchmark_bcd2num(bencher: &mut Bencher) {
    bencher.iter(|| bcd2num(128));
}
 
benchmark_group!(benches_bcd, benchmark_bcd2str, benchmark_bcd2num);
benchmark_main!(benches_bcd);