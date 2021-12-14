use std::env;
use std::net;
use std::str::FromStr;
use env_logger::Builder;
use log::LevelFilter;
use chrono::Local;

pub fn getenv(key: &str, default: &str) -> String {
    let mut rt: String = String::from(default);
    match env::var(key) {
        Ok(v) => (rt = v),
        Err(_) => (),
    }
    rt
}
pub fn str2ip(str: &str) -> net::SocketAddr {
    net::SocketAddr::from(net::SocketAddrV4::from_str(str).unwrap())
}

pub fn setup_log() {
    Builder::new()
    .format(|buf, record| {
        writeln!(
            buf,
            "{} [{}] - {}",
            Local::now().format("%Y-%m-%dT%H:%M:%S.%f"),
            record.level(),
            record.args()
        )
    })
    .filter(None, LevelFilter::Error)
    .init();
}


#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test]
    fn str2ip_test() {
        assert_eq!(
            str2ip("224.0.100.100:10000"),
            net::SocketAddr::new(
                net::IpAddr::from(net::Ipv4Addr::new(224, 0, 100, 100)),
                10000
            )
        )
    }

    #[test_case(
        "192.168.32.23:10000", 
        net::SocketAddr::new(net::IpAddr::from(net::Ipv4Addr::new(192, 168, 32, 23)), 10000); 
    "case1")]
    fn str2ip_testcase(ip_str: &str, addr: net::SocketAddr) {
        assert_eq!(str2ip(ip_str), addr);
    }
}
