use local_ip_address::list_afinet_netifas;
use log::{error, info};
use rand::Rng;
use std::net::{IpAddr, Ipv6Addr};

pub fn get_ipv6(network_name: &str) -> Option<Ipv6Addr> {
    let network_interfaces = list_afinet_netifas();

    let network_interfaces = match network_interfaces {
        Ok(network_interfaces) => network_interfaces,
        Err(e) => {
            error!("Error getting network interfaces: {:?}", e);
            return None;
        }
    };
    let global_ipv6_addrs: Vec<_> = network_interfaces
        .iter()
        .filter(|(name, ip)| {
            if network_name != name {
                return false;
            }
            let ip = match ip {
                IpAddr::V4(_) => {
                    return false;
                }
                IpAddr::V6(ip) => ip,
            };
            if ip.is_loopback() {
                return false;
            }
            if ip.is_unicast_link_local() {
                return false;
            }
            true
        })
        .collect();
    // For windows and linux, the first ip is not temporary.
    let temporary_global_ipv6_addrs: Vec<_> = global_ipv6_addrs.iter().skip(1).collect();
    let mut rng = rand::rng();
    let random_ip =
        temporary_global_ipv6_addrs[rng.random_range(..temporary_global_ipv6_addrs.len())];
    info!("Select: {}: {:?}", random_ip.0, random_ip.1);
    match random_ip.1 {
        IpAddr::V4(_) => None,
        IpAddr::V6(ip) => Some(ip),
    }
}
