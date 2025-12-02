use std::net::Ipv6Addr;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Ipv6AddrInfo {
    pub network_name: String,
    pub address: Ipv6Addr,
    pub valid_lifetime: Duration,
    pub preferred_lifetime: Duration,
    pub address_type: AddressType,
}

#[derive(Debug, Clone)]
pub enum AddressType {
    Loopback,
    LinkLocal,
    Local,
    Normal,
    Temporary,
    Unspecified,
    Other,
}
