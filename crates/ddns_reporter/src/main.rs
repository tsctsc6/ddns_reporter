mod application;
mod client;
mod command;
mod config;
mod get_ipv6_details;
mod logger;
mod netwatcher_filter;
mod reporters;

use std::{process::ExitCode, sync::Arc};

use crate::{
    client::init_client,
    config::init_config,
    logger::init_logger,
    netwatcher_filter::{init_observer, is_send_event},
};
use clap::Parser;
use rxrust::prelude::Observer;
use tracing::{error, info};

fn main() -> ExitCode {
    let cli = command::Cli::parse();

    if let Err(e) = init_logger(cli.verbose) {
        eprintln!("{:?}", e);
        return ExitCode::FAILURE;
    }

    info!("App started");

    let config = match init_config() {
        Ok(config) => Arc::new(config),
        Err(e) => {
            error!("{:?}", e);
            return ExitCode::FAILURE;
        }
    };

    let client = init_client();

    let mut watch = match netwatcher::watch_interfaces_blocking() {
        Ok(watch) => watch,
        Err(e) => {
            error!("{:?}", e);
            return ExitCode::FAILURE;
        }
    };
    let mut observer = init_observer(Arc::clone(&config), &client);

    loop {
        let update = watch.changed();
        if is_send_event(&update, &config) {
            observer.next(());
        }
    }

    // ExitCode::SUCCESS;
}
