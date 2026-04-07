use crate::get_ipv6_details::error::Error;
use crate::get_ipv6_details::ipv6addr_info::{AddressType, Ipv6AddrInfo};
use scopeguard::defer;
use std::net::Ipv6Addr;
use std::str::FromStr;
use std::time::Duration;
use windows::Win32::Foundation::{ERROR_BUFFER_OVERFLOW, ERROR_SUCCESS, NO_ERROR};
use windows::Win32::NetworkManagement::IpHelper::{
    GAA_FLAG_INCLUDE_PREFIX, GetAdaptersAddresses, IP_ADAPTER_ADDRESSES_LH,
    IP_ADAPTER_UNICAST_ADDRESS_LH,
};
use windows::Win32::Networking::WinSock;
use windows::Win32::Networking::WinSock::{AF_INET6, SOCKADDR_IN6};
use windows::Win32::Networking::WinSock::{WSACleanup, WSADATA, WSAStartup};
use windows::core::{HRESULT, PWSTR};

// https://docs.rs/windows/latest/windows/
// https://learn.microsoft.com/zh-cn/windows/win32/api/

#[cfg(target_os = "windows")]
pub fn get_ipv6_addr_info(specified_network_name: &str) -> Result<Vec<Ipv6AddrInfo>, Error> {
    unsafe {
        let mut wsa_data = WSADATA::default();
        // use Winsock v2.2
        // rc is the i32 Win32 error code
        let rc = WSAStartup(0x0202u16, &mut wsa_data);
        if rc != 0 {
            Err(Error::WindowsError(windows::core::Error::new(
                HRESULT(rc),
                "WSAStartup failed",
            )))?;
        }
        defer! {
            WSACleanup();
        }

        let family = AF_INET6.0 as u32; // only get IPv6 addresses
        let flags = GAA_FLAG_INCLUDE_PREFIX; // include prefix information to get lifecycle details
        let mut buffer_length: u32 = 0;

        // First call: get required buffer size
        // rc is the u32 Win32 error code
        let rc = GetAdaptersAddresses(family, flags, None, None, &mut buffer_length);

        if rc != ERROR_BUFFER_OVERFLOW.0 {
            Err(Error::WindowsError(windows::core::Error::new(
                HRESULT::from_win32(rc),
                "GetAdaptersAddresses failed",
            )))?;
        }

        // malloc buffer for adapter addresses
        let mut buffer = vec![0u8; buffer_length as usize];
        let adapter_addresses = buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH;

        // Second call: fill the buffer
        let rc = GetAdaptersAddresses(
            family,
            flags,
            None,
            Some(adapter_addresses),
            &mut buffer_length,
        );

        if rc != NO_ERROR.0 && rc != ERROR_SUCCESS.0 {
            Err(Error::WindowsError(windows::core::Error::new(
                HRESULT::from_win32(rc),
                "GetAdaptersAddresses failed",
            )))?;
        }

        Ok(handle_adapters(adapter_addresses, specified_network_name)?)
    }
}

#[cfg(target_os = "windows")]
fn handle_adapters(
    adapter_addresses: *mut IP_ADAPTER_ADDRESSES_LH,
    specified_network_name: &str,
) -> Result<Vec<Ipv6AddrInfo>, Error> {
    unsafe {
        let mut result: Vec<Ipv6AddrInfo> = vec![];

        // Iterate through adapter list
        let mut current_adapter = adapter_addresses;
        while !current_adapter.is_null() {
            // Get the network name for the adapter
            let adapter_network_name = (*current_adapter).FriendlyName.to_string()?;
            if !str::is_empty(specified_network_name)
                && adapter_network_name != specified_network_name
            {
                current_adapter = (*current_adapter).Next;
                continue;
            }
            handle_adapter(current_adapter, &mut result, adapter_network_name.as_str())?;
            current_adapter = (*current_adapter).Next;
        }
        Ok(result)
    }
}

#[cfg(target_os = "windows")]
fn handle_adapter(
    current_adapter: *mut IP_ADAPTER_ADDRESSES_LH,
    result: &mut Vec<Ipv6AddrInfo>,
    adapter_network_name: &str,
) -> Result<(), Error> {
    unsafe {
        // Iterate through the unicast address list
        let mut unicast_address = (*current_adapter).FirstUnicastAddress;
        while !unicast_address.is_null() {
            // Confirm this is an IPv6 address
            let socket_address = (*unicast_address).Address.lpSockaddr as *const SOCKADDR_IN6;
            if (*socket_address).sin6_family != AF_INET6 {
                unicast_address = (*unicast_address).Next;
                continue;
            }
            result.push(handle_address(unicast_address, adapter_network_name)?);
            unicast_address = (*unicast_address).Next;
        }
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn handle_address(
    unicast_address: *mut IP_ADAPTER_UNICAST_ADDRESS_LH,
    adapter_network_name: &str,
) -> Result<Ipv6AddrInfo, Error> {
    unsafe {
        // Get the IPv6 address string
        let mut ip_str = [0u16; 64]; // Maximum IPv6 address length
        let ip_ptr = PWSTR(ip_str.as_mut_ptr());
        let rc = WinSock::WSAAddressToStringW(
            (*unicast_address).Address.lpSockaddr,
            (*unicast_address).Address.iSockaddrLength as u32,
            None,
            ip_ptr,
            &mut 64,
        );
        if rc != 0 {
            Err(Error::WindowsError(windows::core::Error::new(
                HRESULT::from_win32(rc as u32),
                "GetAdaptersAddresses failed",
            )))?;
        }

        let ip_addr = ip_ptr.to_string()?;
        let ip_addr_and_scope = if let Some((ip, scope)) = ip_addr.split_once('%') {
            let addr = Ipv6Addr::from_str(ip)?;
            let scope_id = scope.parse::<u32>().ok();
            (addr, scope_id)
        } else {
            (Ipv6Addr::from_str(ip_addr.as_str())?, None)
        };
        let ip_addr = ip_addr_and_scope.0;

        // Get lifetimes (in seconds)
        let preferred_lifetime = (*unicast_address).PreferredLifetime;
        let valid_lifetime = (*unicast_address).ValidLifetime;
        let preferred_lifetime = Duration::from_secs(preferred_lifetime as u64);
        let valid_lifetime = Duration::from_secs(valid_lifetime as u64);
        let _prefix_origin = (*unicast_address).PrefixOrigin.0;
        let suffix_origin = (*unicast_address).SuffixOrigin.0;
        //let origin = ((prefix_origin as u8) << 4) + (suffix_origin as u8);

        // Determine IPv6 address type
        // https://learn.microsoft.com/zh-cn/windows/win32/api/nldef/ne-nldef-nl_prefix_origin
        // https://learn.microsoft.com/zh-cn/windows/win32/api/nldef/ne-nldef-nl_suffix_origin
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
        } else if suffix_origin == 5 {
            AddressType::Temporary
        } else {
            AddressType::Normal
        };

        Ok(Ipv6AddrInfo {
            network_name: adapter_network_name.to_string(),
            address: ip_addr,
            valid_lifetime,
            preferred_lifetime,
            address_type,
        })
    }
}
