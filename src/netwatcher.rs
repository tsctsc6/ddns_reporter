use std::{sync::OnceLock, time::Duration};

use crate::application::report_all;
use crate::debounce_manager::DebounceManager;
use crate::reporters::create_reporter;
use netwatcher::WatchHandle;
use std::sync::Arc;
use thiserror::Error;
use tokio::runtime::Handle;
use tokio::task::block_in_place;
use tracing::{debug, error, info};

use crate::config::get_config;

static NETWATCHER_HANDLER: OnceLock<WatchHandle> = OnceLock::new();

#[derive(Error, Debug)]
pub enum Error {
    #[error("Netwatcher error:\n{0}")]
    Netwatcher(#[from] netwatcher::Error),
    #[error("Failed to initialize netwatcher")]
    NetwatcherInitializationError(),
}

pub fn init_netwatcher() -> Result<(), Error> {
    let reporters = Arc::new(create_reporter(get_config()));

    let debouncer = Arc::new(DebounceManager::new(
        {
            let reporters = Arc::clone(&reporters);
            move || {
                let reporters = Arc::clone(&reporters);
                async move {
                    match report_all(reporters).await {
                        Ok(_) => {}
                        Err(e) => error!("{:?}", e),
                    };
                }
            }
        },
        Duration::from_millis(get_config().debounce_time_in_ms),
    ));
    let debouncer_clone = Arc::clone(&debouncer);
    let rt_handle = Handle::current().clone();

    let netwatcher_handler = netwatcher::watch_interfaces(move |update| {
        let network_name = get_config().network_name.clone();
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
                block_in_place(|| rt_handle.block_on(debouncer_clone.trigger()))
            } else {
                // If not, directly block_on with the cloned handle
                rt_handle.block_on(debouncer_clone.trigger());
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
            block_in_place(|| rt_handle.block_on(debouncer_clone.trigger()))
        } else {
            // If not, directly block_on with the cloned handle
            rt_handle.block_on(debouncer_clone.trigger());
        }
    })?;

    NETWATCHER_HANDLER
        .set(netwatcher_handler)
        .map_err(|_| Error::NetwatcherInitializationError())?;

    Ok(())
}
