use futures::StreamExt;
use if_watch::smol::IfWatcher;
use if_watch::{IfEvent, IpNet};

#[tokio::main]
async fn main() {
    let mut set = IfWatcher::new().unwrap();
    loop {
        let event = set.select_next_some().await;
        let event = match event {
            Ok(r) => r,
            Err(_) => continue,
        };
        let ip = match event {
            IfEvent::Up(ip) => ip,
            IfEvent::Down(_) => continue,
        };
        let ip = match ip {
            IpNet::V4(_) => continue,
            IpNet::V6(ipv6) => ipv6,
        };
        if ip.addr().is_loopback() {
            continue;
        }
        if ip.addr().is_unique_local() {
            continue;
        }
        println!("{}", ip);
    }
}
