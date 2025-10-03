use crate::reporters::report_error::{ReportError, SimpleError};
use crate::reporters::reporter::Reporter;
use async_trait::async_trait;
use log::debug;
use reqwest::Client;
use serde_json::Value;
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

#[async_trait]
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
        let mut payload = serde_json::from_str::<Value>(&payload)?;
        let payload = &mut payload["result"];
        let payload = match payload.as_object_mut() {
            Some(x) => x,
            None => {
                return Err(ReportError::Business(SimpleError::from(
                    "Failed to parse response".to_string(),
                )));
            }
        };
        match payload.remove("created_on") {
            Some(x) => x,
            None => {
                return Err(ReportError::Business(SimpleError::from(
                    "Failed to remove created_on from response".to_string(),
                )));
            }
        };
        match payload.remove("modified_on") {
            Some(x) => x,
            None => {
                return Err(ReportError::Business(SimpleError::from(
                    "Failed to remove modified_on from response".to_string(),
                )));
            }
        };
        let ip_value = match payload.get_mut("content") {
            Some(x) => x,
            None => {
                return Err(ReportError::Business(SimpleError::from(
                    "Failed to get content from response".to_string(),
                )));
            }
        };
        debug!("origin ip: {}", ip_value);
        *ip_value = Value::String(ipv6addr.to_string());
        let payload = serde_json::to_string(&payload)?;
        let response = self
            .client
            .patch(format!(
                "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
                self.zone_id, self.dns_record_id
            ))
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", self.token),
            )
            .body(payload)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(ReportError::Business(SimpleError::from(
                response.text().await?,
            )));
        }
        debug!("response: {:#?}", response.text().await?);
        Ok(())
    }
}
