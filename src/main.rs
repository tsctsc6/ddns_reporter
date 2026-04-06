mod command;
mod config;
mod debounce_manager;
mod get_ipv6_details;
mod logger;
mod reporters;

use crate::config::AppConfig;
use crate::debounce_manager::DebounceManager;
use crate::get_ipv6_details::get_ipv6_addr_info;
use crate::get_ipv6_details::ipv6addr_info::AddressType;
use crate::logger::init_logger;
use crate::reporters::reporter::create_reporter;
use ::config::{Config, File as ConfigFile};
use clap::Parser;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Handle;
use tokio::task::block_in_place;
use tokio::time::sleep;
use tracing::{debug, error, info};

#[tokio::main]
async fn main() -> Result<(), i32> {
    let cli = command::Cli::parse();
    let builder = Config::builder().add_source(ConfigFile::with_name("config.toml"));
    let config = builder.build();
    let config = match config {
        Ok(c) => c,
        Err(e) => {
            error!("{:?}", e);
            return Err(1);
        }
    };
    let app_config: AppConfig = match config.try_deserialize::<AppConfig>() {
        Ok(config) => config,
        Err(e) => {
            error!("{:?}", e);
            return Err(1);
        }
    };

    match init_logger(cli.verbose) {
        Ok(_) => {}
        Err(e) => {
            error!("{:?}", e);
            return Err(1);
        }
    }

    info!("App started");

    let reporter = create_reporter(&app_config);
    let network_name = app_config.network_name.clone();
    let retry_count = app_config.retry_count;
    let retry_interval_in_second = app_config.retry_interval_in_second;

    let closer = {
        let reporter = Arc::clone(&reporter);
        let network_name = network_name.clone();
        let retry_count = retry_count;
        let retry_interval_in_second = retry_interval_in_second;
        move || {
            let reporter = Arc::clone(&reporter);
            let network_name = network_name.clone();
            let retry_count = retry_count;
            let retry_interval_in_second = retry_interval_in_second;

            async move {
                let mut wait_time_second = 1u64;
                for current_retry_count in 0..retry_count {
                    info!("Retry: {}", current_retry_count);
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
                    sleep(Duration::from_secs(wait_time_second)).await;
                    match reporter.report(ipv6.address).await {
                        Ok(_) => {
                            info!("Report complete: {}", ipv6.address);
                            break;
                        }
                        Err(e) => {
                            error!("{:?}", e);
                            wait_time_second = wait_time_second * 2;
                            if wait_time_second >= retry_interval_in_second {
                                wait_time_second = retry_interval_in_second;
                            }
                        }
                    };
                }
            }
        }
    };

    let debouncer = Arc::new(DebounceManager::new(
        closer,
        Duration::from_millis(app_config.debounce_time_in_ms),
    ));
    let debouncer_arc = Arc::clone(&debouncer);
    let rt_handle = Handle::current().clone();

    let register_handler_result = netwatcher::watch_interfaces(move |update| {
        let network_name = network_name.clone();
        let network_index = update
            .interfaces
            .iter()
            .filter(|x| -> bool { x.1.name == network_name })
            .map(|x| -> u32 { *x.0 })
            .next();
        let network_index = match network_index {
            None => {
                error!("Interface {} not found", network_name);
                return;
            }
            Some(i) => {
                info!("Interface {} found: {}", network_name, i);
                i
            }
        };

        debug!("Interfaces added: {:?}", update.diff.added);
        debug!("Interfaces removed: {:?}", update.diff.removed);

        if update.diff.added.contains(&network_index) {
            // Check if we're currently in a Tokio runtime context
            if Handle::try_current().is_ok() {
                // If yes, use block_in_place to temporarily yield the async context
                block_in_place(|| rt_handle.block_on(debouncer_arc.trigger()))
            } else {
                // If not, directly block_on with the cloned handle
                rt_handle.block_on(debouncer_arc.trigger());
            }
            return;
        }

        debug!("{:?}", update.diff.modified);

        let network_diff = update
            .diff
            .modified
            .iter()
            .filter(|x| -> bool { *x.0 == network_index })
            .next();
        let network_diff = match network_diff {
            None => {
                debug!("Interface {} not found in network_diff", network_name);
                return;
            }
            Some(d) => d,
        };
        debug!("Interface index {} has changed", network_diff.0);
        debug!("Added IPs: {:?}", network_diff.1.addrs_added);
        debug!("Removed IPs: {:?}", network_diff.1.addrs_removed);
        // Check if we're currently in a Tokio runtime context
        if Handle::try_current().is_ok() {
            // If yes, use block_in_place to temporarily yield the async context
            block_in_place(|| rt_handle.block_on(debouncer_arc.trigger()))
        } else {
            // If not, directly block_on with the cloned handle
            rt_handle.block_on(debouncer_arc.trigger());
        }
    });
    let _register_handler = match register_handler_result {
        Ok(h) => h,
        Err(e) => {
            error!("{:?}", e);
            return Err(1);
        }
    };

    loop {
        sleep(Duration::MAX).await;
    }

    // Ok(())
}
