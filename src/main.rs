mod config;
mod debounce_manager;
mod get_ipv6;
mod reporters;

use crate::config::AppConfig;
use crate::debounce_manager::DebounceManager;
use crate::get_ipv6::get_ipv6;
use crate::reporters::reporter::{Reporter, create_reporter};
use ::config::{Config, File};
use futures::StreamExt;
use if_watch::smol::IfWatcher;
use log::{error, info};
use std::env;
use std::process::Termination;
use std::sync::{Arc, LockResult, Mutex, MutexGuard};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let builder = Config::builder().add_source(File::with_name("config.toml"));
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

    // 设置日志级别
    unsafe {
        env::set_var("RUST_LOG", "debug");
    }
    // 初始化 env_logger
    env_logger::init();

    let reporter = create_reporter(app_config);

    let debouncer: DebounceManager<dyn Reporter> =
        DebounceManager::new(Duration::from_secs(1), String::from("以太网"), reporter);

    let mut set = match IfWatcher::new() {
        Ok(set) => set,
        Err(e) => {
            error!("{:?}", e);
            return;
        }
    };
    loop {
        let event = set.select_next_some().await;
        debouncer.trigger().await;
        info!("{:?}", event);
    }
}
