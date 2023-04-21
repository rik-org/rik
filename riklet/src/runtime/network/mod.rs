pub mod function_network;
pub mod pod_network;

use async_trait::async_trait;
use once_cell::sync::Lazy;
use shared::utils::ip_allocator::IpAllocator;
use std::fmt::Debug;
use std::sync::Mutex;
use thiserror::Error;

use crate::cli::function_config::FnConfiguration;
use crate::iptables::rule::Rule;
use crate::iptables::{Chain, Iptables, IptablesError, MutateIptables, Table};
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

    #[error("Should have been able to apply a valid IP address to the interface, but failed: {0}")]
    InterfaceIPError(String),
}

type Result<T> = std::result::Result<T, NetworkError>;

#[async_trait]
pub trait RuntimeNetwork: Send + Sync {
    async fn init(&mut self) -> Result<()>;

    /// Called after the workload has been created in the system but not booted
    async fn preboot(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called after the workload has been running in the system
    async fn postboot(&mut self) -> Result<()> {
        Ok(())
    }

    async fn destroy(&self) -> Result<()>;
}

pub struct GlobalRuntimeNetwork {
    function_config: FnConfiguration,
}

impl GlobalRuntimeNetwork {
    pub fn new() -> Self {
        GlobalRuntimeNetwork {
            function_config: FnConfiguration::load(),
        }
    }
}

#[async_trait]
impl RuntimeNetwork for GlobalRuntimeNetwork {
    async fn init(&mut self) -> Result<()> {
        let mut ipt = Iptables::new(false).map_err(NetworkError::IptablesError)?;

        // Allow NAT on the interface connected to the internet.
        let rule = Rule {
            rule: format!("-o {} -j MASQUERADE", self.function_config.ifnet),
            chain: Chain::PostRouting,
            table: Table::Nat,
        };
        ipt.create(&rule).map_err(NetworkError::IptablesError)?;

        // Add the FORWARD rules to the filter table
        let rule = Rule {
            rule: "-m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT".to_string(),
            chain: Chain::Forward,
            table: Table::Filter,
        };
        ipt.create(&rule).map_err(NetworkError::IptablesError)?;

        Ok(())
    }

    async fn destroy(&self) -> Result<()> {
        Ok(())
    }
}
