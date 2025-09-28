use crate::reporters::report_error::{ReportError, SimpleError};
use crate::reporters::reporter::Reporter;
use reqwest::Client;
use serde_json::json;
use std::net::Ipv6Addr;

pub struct CloudflareReporter {
    client: Client,
    zone_id: String,
    dns_record_id: String,
    token: String,
}

impl CloudflareReporter {
    pub fn new(zone_id: &str, dns_record_id: &str, token: &str) -> CloudflareReporter {
        CloudflareReporter {
            client: Client::new(),
            zone_id: String::from(zone_id),
            dns_record_id: String::from(dns_record_id),
            token: String::from(token),
        }
    }
}

impl Reporter for CloudflareReporter {
    async fn report(&self, ipv6addr: Ipv6Addr) -> Result<(), ReportError> {
        let response = self
            .client
            .get(format!(
                "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
                self.zone_id, self.dns_record_id
            ))
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", self.token),
            )
            .send()
            .await?;
        // 构建 JSON 数据
        let payload = response.text().await?;
        let mut payload = serde_json::from_str::<serde_json::Value>(&payload).unwrap();
        let mut payload = &mut payload["result"];
        let mut payload = payload.as_object_mut().unwrap();
        payload.remove("created_on").unwrap();
        payload.remove("modified_on").unwrap();
        let payload = serde_json::to_string_pretty(&payload).unwrap();
        let response = self
            .client
            .get(format!(
                "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
                self.zone_id, self.dns_record_id
            ))
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", self.token),
            )
            .json(payload.as_str())
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(ReportError::Business(SimpleError::from(
                response.text().await?,
            )));
        }
        Ok(())
    }
}
