mod config;
mod reporters;

use crate::config::AppConfig;
use crate::reporters::reporter::{Reporter, create_reporter};
use ::config::{Config, File};
use futures::StreamExt;
use if_watch::smol::IfWatcher;
use if_watch::{IfEvent, IpNet};

#[tokio::main]
async fn main() {
    let builder = Config::builder().add_source(File::with_name("config.toml"));
    let config = builder.build();
    let config = match config {
        Ok(c) => c,
        Err(e) => {
            println!("{:?}", e);
            return;
        }
    };
    let app_config: AppConfig = match config.try_deserialize::<AppConfig>() {
        Ok(config) => config,
        Err(e) => {
            println!("{:?}", e);
            return;
        }
    };
    println!("{:#?}", app_config);
    let reporter = create_reporter(app_config);

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
        let report_result = reporter.report(ip.addr()).await;
        match report_result {
            Ok(_) => {}
            Err(e) => {
                println!("{:#?}", e);
            }
        }
        println!("{}", ip);
    }
}
