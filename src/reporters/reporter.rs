use crate::config::{AppConfig, DdnsServer};
use crate::reporters::cloudflare_reporter::CloudflareReporter;
use crate::reporters::report_error::ReportError;
use async_trait::async_trait;
use std::net::Ipv6Addr;
use std::sync::Arc;

#[async_trait]
pub trait Reporter: Send + Sync {
    async fn report(&self, ipv6addr: Ipv6Addr) -> Result<(), ReportError>; // 方法签名
}

pub fn create_reporter(app_config: &AppConfig) -> Arc<dyn Reporter> {
    match app_config.ddns_server {
        DdnsServer::Cloudflare => Arc::new(CloudflareReporter::new(
            &app_config.cloudflare.zone_id,
            &app_config.cloudflare.dns_record_id,
            &app_config.cloudflare.token,
        )),
    }
}
