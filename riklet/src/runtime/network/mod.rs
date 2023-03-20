pub mod function_network;
pub mod pod_network;

use async_trait::async_trait;
use once_cell::sync::Lazy;
use shared::utils::ip_allocator::IpAllocator;
use std::fmt::Debug;
use std::sync::Mutex;
use thiserror::Error;

use crate::iptables::IptablesError;
use crate::network::net::NetworkInterfaceError;

// Initialize Singleton for IpAllocator
static IP_ALLOCATOR: Lazy<Mutex<IpAllocator>> = Lazy::new(|| {
    let ip_allocator = IpAllocator::new().expect("Fail to load IP allocator");
    Mutex::new(ip_allocator)
});

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Network error: {0}")]
    Error(String),

    #[error("Iptables error: {0}")]
    IptablesError(IptablesError),

    #[error("Parsing error: {0}")]
    ParsingError(serde_json::Error),

    #[error("Network interface error: {0}")]
    NetworkInterfaceError(NetworkInterfaceError),
}

type Result<T> = std::result::Result<T, NetworkError>;

#[async_trait]
pub trait RuntimeNetwork: Send + Sync + Debug {
    async fn init(&self) -> Result<()>;

    async fn destroy(&self) -> Result<()>;
}
