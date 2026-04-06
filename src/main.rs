mod application;
mod command;
mod config;
mod debounce_manager;
mod get_ipv6_details;
mod logger;
mod netwatcher;
mod reporters;

use std::time::Duration;

use crate::{config::init_config, logger::init_logger};
use clap::Parser;
use tokio::time::sleep;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), i32> {
    let cli = command::Cli::parse();

    if let Err(e) = init_config() {
        error!("{:?}", e);
        return Err(1);
    }

    if let Err(e) = init_logger(cli.verbose) {
        error!("{:?}", e);
        return Err(1);
    }

    info!("App started");

    if let Err(e) = netwatcher::init_netwatcher() {
        error!("{:?}", e);
        return Err(1);
    }

    loop {
        sleep(Duration::MAX).await;
    }

    // Ok(())
}
