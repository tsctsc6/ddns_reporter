use std::time::Duration;

use crate::reporters::create_reporter;
use crate::{application::report_all, config::AppConfig};
use reqwest::Client;
use rxrust::prelude::Publisher;
use rxrust::{
    observable,
    prelude::{Observable, SubscribeNext},
    shared::SharedObservable,
    subject::SharedSubject,
};
use std::sync::Arc;
use tracing::{debug, error, info};

pub fn init_observer(
    config: Arc<AppConfig>,
    client: &Client,
) -> rxrust::prelude::Subject<
    Arc<std::sync::Mutex<Vec<Box<dyn Publisher<Item = (), Err = ()> + Send + Sync + 'static>>>>,
    rxrust::prelude::SharedSubscription,
> {
    let reporters = Arc::new(create_reporter(&config, client));
    let scheduler = Arc::new(tokio::runtime::Runtime::new().expect("Failed to create runtime"));
    let scheduler_clone = Arc::clone(&scheduler);
    let subject = SharedSubject::<(), ()>::new();

    subject
        .clone()
        .debounce(
            Duration::from_millis(config.debounce_time_in_ms),
            Arc::clone(&scheduler),
        )
        .flat_map(move |_| {
            observable::from_future(
                report_all(Arc::clone(&reporters), Arc::clone(&config)),
                Arc::clone(&scheduler_clone),
            )
        })
        .into_shared()
        .subscribe(move |x| {
            match x {
                Ok(_) => {}
                Err(e) => error!("{:?}", e),
            };
        });
    subject
}

pub fn is_send_event(update: &netwatcher::Update, config: &AppConfig) -> bool {
    let network_name = config.network_name.clone();
    let network_index = update
        .interfaces
        .iter()
        .filter(|x| -> bool { x.1.name == network_name })
        .map(|x| -> u32 { *x.0 })
        .next();
    let network_index = match network_index {
        None => {
            error!("Interface {} not found", network_name);
            return false;
        }
        Some(i) => {
            info!("Interface {} found: {}", network_name, i);
            i
        }
    };

    debug!("Interfaces added: {:?}", update.diff.added);
    debug!("Interfaces removed: {:?}", update.diff.removed);

    if update.diff.added.contains(&network_index) {
        return true;
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
            return false;
        }
        Some(d) => d,
    };
    debug!("Interface index {} has changed", network_diff.0);
    debug!("Added IPs: {:?}", network_diff.1.addrs_added);
    debug!("Removed IPs: {:?}", network_diff.1.addrs_removed);
    return true;
}
