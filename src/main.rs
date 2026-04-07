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
use tracing::{error, info};

fn main() -> Result<(), i32> {
    let cli = command::Cli::parse();

    if let Err(e) = init_logger(cli.verbose) {
        eprintln!("{:?}", e);
        return Err(1);
    }

    info!("App started");

    if let Err(e) = init_config() {
        error!("{:?}", e);
        return Err(1);
    }

    if let Err(e) = netwatcher::init_netwatcher() {
        error!("{:?}", e);
        return Err(1);
    }

    loop {
        std::thread::sleep(Duration::MAX);
    }

    // Ok(())
}
