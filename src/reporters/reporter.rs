use std::net::Ipv6Addr;
use crate::reporters::report_error::ReportError;

pub trait Reporter {
    async fn report(&self, ipv6addr: Ipv6Addr) -> Result<(), ReportError>; // 方法签名
}
