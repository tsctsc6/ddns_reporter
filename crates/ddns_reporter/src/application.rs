use std::{net::Ipv6Addr, sync::Arc, time::Duration};

use thiserror::Error;
use tokio::{task::JoinSet, time::sleep};
use tracing::{debug, error, info};

use crate::{
    config::AppConfig,
    get_ipv6_details::{get_ipv6_addr_info, ipv6addr_info::AddressType},
    reporters::Reporter,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Get IPv6 address info error:\n{0}")]
    GetIpv6AddrInfoError(#[from] crate::get_ipv6_details::error::Error),
    #[error("Temporary IP address not found")]
    TemporaryIPAddressNotFound(),
}

pub async fn report_all(
    reporters: Arc<Vec<Arc<dyn Reporter>>>,
    config: Arc<AppConfig>,
) -> Result<(), Error> {
    let network_name = config.network_name.clone();
    let ipv6_list = get_ipv6_addr_info(network_name.as_str()).await?;
    debug!("IP lists found: {:#?}", ipv6_list);
    let ipv6 = ipv6_list
        .iter()
        .filter(|x| -> bool { x.network_name == network_name })
        .filter(|x| -> bool { x.address_type == AddressType::Temporary })
        .max_by_key(|x| -> Duration { x.preferred_lifetime });
    let ipv6 = match ipv6 {
        None => {
            return Err(Error::TemporaryIPAddressNotFound());
        }
        Some(ipv6) => ipv6.clone(),
    };
    debug!("IP found: {:#?}", ipv6);

    let mut set = JoinSet::new();
    for reporter in reporters.iter() {
        let config_clone = Arc::clone(&config);
        let reporter = Arc::clone(reporter);
        set.spawn(async move { report_one(reporter, &config_clone, ipv6.address).await });
    }
    set.join_all().await;
    Ok(())
}

async fn report_one(reporter: Arc<dyn Reporter>, config: &AppConfig, ipv6addr: Ipv6Addr) -> () {
    let mut wait_time_second = 1u64;
    let retry_count = config.retry_count;
    let retry_interval_in_second = config.retry_interval_in_second;
    for current_retry_count in 0..retry_count {
        debug!("Retry: {}", current_retry_count);
        match reporter.report(ipv6addr).await {
            Ok(_) => {
                info!("Report complete: {}", ipv6addr);
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
        sleep(Duration::from_secs(wait_time_second)).await;
    }
}
