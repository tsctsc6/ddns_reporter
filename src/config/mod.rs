use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub ddns_server: DdnsServer,
    pub network_name: String,
    pub log_level: LogLevel,
    pub debounce_time_in_ms: u64,
    pub cloudflare: CloudflareConfig,
    pub retry_count: u64,
    pub retry_interval_in_second: u64,
}

#[derive(Debug, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
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
