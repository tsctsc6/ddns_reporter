#[cfg(target_os = "windows")]
use std::net::AddrParseError;
#[cfg(target_os = "windows")]
use std::string::FromUtf16Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Universal error: {0}")]
    SimpleError(#[from] UniversalError),
    #[cfg(target_os = "windows")]
    #[error("From UTF16 error: {0}")]
    FromUtf16Error(#[from] FromUtf16Error),
    #[cfg(target_os = "windows")]
    #[error("Ip addr parse error: {0}")]
    IpAddrParseError(#[from] AddrParseError),
    #[cfg(target_os = "windows")]
    #[error("Windows error: {0}")]
    WindowsError(#[from] windows::core::Error),
    #[cfg(target_os = "linux")]
    #[error("Linux error: {0}")]
    NetlinkError(#[from] rtnetlink::Error),
}

#[derive(Debug, Error)]
#[error("[{code}] {message}")]
pub struct UniversalError {
    pub code: i32,
    pub message: String,
}
