use crate::get_ipv6_details::error::{Error, UniversalError};
use crate::get_ipv6_details::ipv6addr_info::{AddressType, Ipv6AddrInfo};
use core::clone::Clone;
use futures::TryStreamExt;
use netlink_packet_route::AddressFamily;
use netlink_packet_route::address::{AddressAttribute, AddressFlags, AddressMessage};
use netlink_packet_route::link::LinkAttribute;
use rtnetlink::new_connection;
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::Duration;
use tokio::runtime::Handle;
use tokio::task::block_in_place;

#[cfg(target_os = "linux")]
pub fn get_ipv6_addr_info(specified_network_name: &str) -> Result<Vec<Ipv6AddrInfo>, Error> {
    let rt_handle = Handle::current().clone();
    // Check if we're currently in a Tokio runtime context
    if Handle::try_current().is_ok() {
        // If yes, use block_in_place to temporarily yield the async context
        block_in_place(|| rt_handle.block_on(get_ipv6_addr_info2(specified_network_name)))
    } else {
        // If not, directly block_on with the cloned handle
        rt_handle.block_on(get_ipv6_addr_info2(specified_network_name))
    }
}

#[cfg(target_os = "linux")]
async fn get_ipv6_addr_info2(specified_network_name: &str) -> Result<Vec<Ipv6AddrInfo>, Error> {
    let mut result: Vec<Ipv6AddrInfo> = vec![];
    // 创建 Netlink 连接和句柄
    let (connection, handle, _) = new_connection().map_err(|e| format!("{:?}", e)).unwrap();
    tokio::spawn(connection);

    let mut target_link_index = 0;
    let mut links: HashMap<u32, String> = HashMap::new();
    let mut link_messages = handle.link().get().execute();
    while let Some(msg) = link_messages.try_next().await? {
        // println!("msg: {:#?}", msg);
        for attr in msg.attributes {
            if let LinkAttribute::IfName(name) = attr {
                if !str::is_empty(specified_network_name) && name == specified_network_name {
                    target_link_index = msg.header.index;
                }
                links.insert(msg.header.index, name);
            } else {
                continue;
            }
        }
    }
    if !str::is_empty(specified_network_name) && target_link_index == 0 {
        return Ok(vec![]);
    }

    // 获取所有 IPv6 地址（相当于 `ip -6 addr show`）
    let mut address_messages = handle.address().get().execute();

    while let Some(msg) = address_messages.try_next().await? {
        if !str::is_empty(specified_network_name) && msg.header.index != target_link_index {
            continue;
        }
        if msg.header.family != AddressFamily::Inet6 {
            continue;
        }
        //println!("msg: {:#?}", msg);
        let link_name = links.get(&msg.header.index);
        let link_name = match link_name {
            Some(link_name) => link_name,
            None => Err(UniversalError {
                code: 1,
                message: String::from("Link_name not found"),
            })?,
        };
        result.push(handle_address(&msg, link_name)?);
    }

    Ok(result)
}

#[cfg(target_os = "linux")]
fn handle_address(msg: &AddressMessage, adapter_network_name: &str) -> Result<Ipv6AddrInfo, Error> {
    let ip_addr = msg.attributes.iter().find_map(|attr| {
        return if let AddressAttribute::Address(ip_addr) = attr {
            Some(ip_addr)
        } else {
            None
        };
    });
    let ip_addr = match ip_addr {
        Some(ip_addr) => ip_addr,
        None => Err(UniversalError {
            code: 2,
            message: String::from("Address not found"),
        })?,
    };
    let ip_addr = match ip_addr {
        IpAddr::V6(ip_addr) => ip_addr,
        _ => Err(UniversalError {
            code: 3,
            message: String::from("Address is not ipv6"),
        })?,
    };

    let cache_info = msg.attributes.iter().find_map(|attr| {
        return if let AddressAttribute::CacheInfo(cache_info) = attr {
            Some(cache_info)
        } else {
            None
        };
    });
    let cache_info = match cache_info {
        Some(cache_info) => cache_info,
        None => Err(UniversalError {
            code: 4,
            message: String::from("CacheInfo not found"),
        })?,
    };

    let valid_lifetime = Duration::from_secs(cache_info.ifa_valid as u64);
    let preferred_lifetime = Duration::from_secs(cache_info.ifa_preferred as u64);

    let flags = msg.attributes.iter().find_map(|attr| {
        return if let AddressAttribute::Flags(flags) = attr {
            Some(flags)
        } else {
            None
        };
    });
    let flags = match flags {
        Some(flags) => flags,
        None => Err(UniversalError {
            code: 5,
            message: String::from("Flags not found"),
        })?,
    };

    let address_type = if ip_addr.is_loopback() {
        AddressType::Loopback
    } else if ip_addr.is_unicast_link_local() {
        AddressType::LinkLocal
    } else if ip_addr.is_unique_local() {
        AddressType::Local
    } else if ip_addr.is_unspecified() {
        AddressType::Unspecified
    } else if ip_addr.is_multicast() {
        AddressType::Other
    } else if flags.contains(AddressFlags::Secondary) {
        AddressType::Temporary
    } else {
        AddressType::Normal
    };

    Ok(Ipv6AddrInfo {
        network_name: adapter_network_name.to_string(),
        address: *ip_addr,
        valid_lifetime,
        preferred_lifetime,
        address_type,
    })
}
