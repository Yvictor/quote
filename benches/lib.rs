extern crate quote;
 
#[macro_use]
extern crate bencher;
 
use quote::add_two;
 
use bencher::Bencher;
 
fn benchmark(bencher: &mut Bencher) {
    bencher.iter(|| add_two(256));
}
 
benchmark_group!(benches, benchmark);
benchmark_main!(benches);