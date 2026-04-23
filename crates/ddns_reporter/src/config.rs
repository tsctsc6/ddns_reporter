use config::{Config, ConfigError, File as ConfigFile};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub ddns_servers: Vec<DdnsServer>,
    pub network_name: String,
    pub debounce_time_in_ms: u64,
    pub cloudflare: CloudflareConfig,
    pub retry_count: u64,
    pub retry_interval_in_second: u64,
}

#[derive(Debug, Deserialize)]
pub enum DdnsServer {
    Cloudflare,
}

#[derive(Debug, Deserialize)]
pub struct CloudflareConfig {
    pub zone_id: String,
    pub dns_record_id: String,
    pub token: String,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Config error:\n{0}")]
    ConfigError(#[from] ConfigError),
}

pub fn init_config() -> Result<AppConfig, Error> {
    let builder = Config::builder().add_source(ConfigFile::with_name("config.toml"));
    let config = builder.build()?;
    let app_config: AppConfig = config.try_deserialize::<AppConfig>()?;
    Ok(app_config)
}
