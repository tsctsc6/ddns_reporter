mod config;
mod debounce_manager;
mod get_ipv6_details;
mod reporters;

use crate::config::{AppConfig, LogLevel};
use crate::debounce_manager::DebounceManager;
use crate::get_ipv6_details::get_ipv6_addr_info;
use crate::get_ipv6_details::ipv6addr_info::AddressType;
use crate::reporters::reporter::create_reporter;
use ::config::{Config, File as ConfigFile};
use futures::StreamExt;
use if_watch::IfEvent;
use if_watch::smol::IfWatcher;
use log::{error, info};
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tracing_appender::{
    non_blocking,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{Layer, filter, layer::SubscriberExt, util::SubscriberInitExt};

fn setup_logging(log_level: &LogLevel) {
    let log_level = match log_level {
        LogLevel::Trace => filter::LevelFilter::TRACE,
        LogLevel::Debug => filter::LevelFilter::DEBUG,
        LogLevel::Info => filter::LevelFilter::INFO,
        LogLevel::Warn => filter::LevelFilter::WARN,
        LogLevel::Error => filter::LevelFilter::ERROR,
    };

    // 1. 创建文件轮转appender
    let file_appender = RollingFileAppender::new(Rotation::DAILY, "logs", "ddns_reporter.log");

    // 2. 非阻塞写入器（提高性能）
    let (non_blocking_file, _guard) = non_blocking(file_appender);

    // 3. 创建文件输出层
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking_file)
        .with_ansi(false)
        .with_filter(log_level);

    // 4. 创建控制台输出层
    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_ansi(true)
        .with_target(true)
        .with_filter(log_level);

    // 5. 组合所有层并初始化
    tracing_subscriber::registry()
        .with(file_layer)
        .with(stdout_layer)
        .init();

    // 注意：必须保留_guard，否则日志写入会立即停止
    std::mem::forget(_guard);
}

#[tokio::main]
async fn main() {
    let builder = Config::builder().add_source(ConfigFile::with_name("config.toml"));
    let config = builder.build();
    let config = match config {
        Ok(c) => c,
        Err(e) => {
            error!("{:?}", e);
            return;
        }
    };
    let app_config: AppConfig = match config.try_deserialize::<AppConfig>() {
        Ok(config) => config,
        Err(e) => {
            error!("{:?}", e);
            return;
        }
    };

    std::fs::create_dir_all("logs").expect("创建日志目录失败");
    setup_logging(&app_config.log_level);

    info!("App started");

    let reporter = create_reporter(&app_config);
    let network_name = app_config.network_name.clone();

    let closer = {
        let reporter = Arc::clone(&reporter);
        let network_name = network_name.clone();
        move || {
            let reporter = Arc::clone(&reporter);
            let network_name = network_name.clone();
            async move {
                let ipv6_list = get_ipv6_addr_info(network_name.as_str());
                let ipv6_list = match ipv6_list {
                    Ok(ipv6_list) => ipv6_list,
                    Err(e) => {
                        error!("{:?}", e);
                        return;
                    }
                };
                let ipv6 = ipv6_list
                    .iter()
                    .filter(|x| -> bool { x.network_name == network_name })
                    .filter(|x| -> bool { x.address_type == AddressType::Temporary })
                    .max_by_key(|x| -> Duration { x.preferred_lifetime });
                let ipv6 = match ipv6 {
                    None => {
                        error!("Max preferred_lifetime ipv6 not found");
                        return;
                    }
                    Some(ipv6) => ipv6,
                };
                match reporter.report(ipv6.address).await {
                    Ok(_) => info!("Report complete: {}", ipv6.address),
                    Err(e) => error!("{:?}", e),
                };
            }
        }
    };

    let debouncer = DebounceManager::new(
        closer,
        Duration::from_millis(app_config.debounce_time_in_ms),
    );

    let mut set = match IfWatcher::new() {
        Ok(set) => set,
        Err(e) => {
            error!("{:?}", e);
            return;
        }
    };
    loop {
        let event = set.select_next_some().await;
        let event = match event {
            Ok(event) => event,
            Err(_) => continue,
        };
        let event2 = match event {
            IfEvent::Up(event) => event,
            IfEvent::Down(event) => event,
        };
        match event2.network() {
            IpAddr::V6(_) => {}
            _ => {
                continue;
            }
        }
        debouncer.trigger().await;
        info!("{:?}", event);
    }
}
