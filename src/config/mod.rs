use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub ddns_server: DdnsServer,
    pub network_name: String,
    pub cloudflare: CloudflareConfig
}

#[derive(Debug, Deserialize)]
pub enum DdnsServer{
    Cloudflare
}

#[derive(Debug, Deserialize)]
pub struct CloudflareConfig {
    pub zone_id: String,
    pub dns_record_id: String,
    pub token: String,
}
