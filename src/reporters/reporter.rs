use crate::config::{AppConfig, DdnsServer};
use crate::reporters::cloudflare_reporter::CloudflareReporter;
use crate::reporters::report_error::ReportError;
use async_trait::async_trait;
use std::net::Ipv6Addr;
use std::sync::Arc;

#[async_trait]
pub trait Reporter: Send + Sync {
    async fn report(&self, ipv6addr: Ipv6Addr) -> Result<(), ReportError>;
}

pub fn create_reporter(app_config: &AppConfig) -> Vec<Arc<dyn Reporter>> {
    let mut reporters: Vec<Arc<dyn Reporter>> = Vec::new();
    for server in &app_config.ddns_servers {
        let reporter: Arc<dyn Reporter> = match server {
            DdnsServer::Cloudflare => Arc::new(CloudflareReporter::new(
                &app_config.cloudflare.zone_id,
                &app_config.cloudflare.dns_record_id,
                &app_config.cloudflare.token,
            )),
        };
        reporters.push(reporter);
    }
    reporters
}
