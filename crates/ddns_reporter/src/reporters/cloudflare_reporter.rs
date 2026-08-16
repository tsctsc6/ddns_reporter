use crate::reporters::{Error, Reporter};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use std::net::Ipv6Addr;
use tracing::debug;

pub struct CloudflareReporter {
    client: Client,
    zone_id: String,
    dns_record_id: String,
    token: String,
}

impl CloudflareReporter {
    pub fn new(
        zone_id: &str,
        dns_record_id: &str,
        token: &str,
        client: &Client,
    ) -> CloudflareReporter {
        CloudflareReporter {
            client: client.clone(),
            zone_id: String::from(zone_id),
            dns_record_id: String::from(dns_record_id),
            token: String::from(token),
        }
    }
}

#[async_trait]
impl Reporter for CloudflareReporter {
    fn get_name(&self) -> &str {
        "Cloudflare Reporter"
    }

    async fn report(&self, ipv6addr: Ipv6Addr) -> Result<(), Error> {
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
        // Build JSON data
        let payload_string = response.text().await?;
        let mut payload = serde_json::from_str::<Value>(&payload_string)?;
        let payload = &mut payload["result"];
        let payload = match payload.as_object_mut() {
            Some(x) => x,
            None => {
                return Err(Error::JsonTypeError(format!(
                    "Failed to parse response:\r\n{payload_string}"
                )));
            }
        };
        match payload.remove("created_on") {
            Some(x) => x,
            None => {
                return Err(Error::JsonOperateError(format!(
                    "Failed to remove created_on from response",
                )));
            }
        };
        match payload.remove("modified_on") {
            Some(x) => x,
            None => {
                return Err(Error::JsonOperateError(format!(
                    "Failed to remove modified_on from response",
                )));
            }
        };
        let ip_value = match payload.get_mut("content") {
            Some(x) => x,
            None => {
                return Err(Error::JsonOperateError(format!(
                    "Failed to get content from response",
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
            return Err(Error::HttpResponseError(response.text().await?));
        }
        debug!("response: {:#?}", response.text().await?);
        Ok(())
    }
}
