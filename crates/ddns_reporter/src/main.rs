mod application;
mod client;
mod command;
mod config;
mod get_ipv6_details;
mod logger;
mod netwatcher;
mod reporters;

use std::{process::ExitCode, time::Duration};

use crate::{
    client::{get_client, init_client},
    config::{get_config, init_config},
    logger::init_logger,
};
use clap::Parser;
use tracing::{error, info};

fn main() -> ExitCode {
    let cli = command::Cli::parse();

    if let Err(e) = init_logger(cli.verbose) {
        eprintln!("{:?}", e);
        return ExitCode::FAILURE;
    }

    info!("App started");

    if let Err(e) = init_config() {
        error!("{:?}", e);
        return ExitCode::FAILURE;
    }

    if let Err(e) = init_client() {
        error!("{:?}", e);
        return ExitCode::FAILURE;
    }

    if let Err(e) = netwatcher::init_netwatcher(get_config(), get_client()) {
        error!("{:?}", e);
        return ExitCode::FAILURE;
    }

    loop {
        std::thread::sleep(Duration::MAX);
    }

    // ExitCode::SUCCESS;
}
