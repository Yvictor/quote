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

fn benchmark_price2long(bencher: &mut Bencher) {
    bencher.iter(|| price2long([0, 133, 32, 0, 0]));
}

fn benchmark_volume2long(bencher: &mut Bencher) {
    bencher.iter(|| volume2long([0, 133, 32, 0]));
}

benchmark_group!(
    benches_bcd,
    benchmark_bcd2str,
    benchmark_bcd2num,
    benchmark_price2long,
    benchmark_volume2long,
);
benchmark_main!(benches_bcd);
