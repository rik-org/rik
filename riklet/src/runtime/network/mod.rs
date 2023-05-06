pub mod function_network;
pub mod pod_network;

use async_trait::async_trait;
use once_cell::sync::Lazy;
use shared::utils::ip_allocator::IpAllocator;
use std::fmt::Debug;
use std::net::Ipv4Addr;
use std::sync::Mutex;
use thiserror::Error;

use crate::iptables::rule::Rule;
use crate::iptables::{Chain, Iptables, IptablesError, MutateIptables, Table};

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

    async fn destroy(&mut self) -> Result<()>;
}

pub struct GlobalRuntimeNetwork {
    /// Unique instance of iptables which contain all rules and chains generated
    /// for the global configuration of the network
    iptables: Iptables,

    /// Name of the interface that will be used as the gateway for the network
    gateway_iface: Ipv4Addr,
}

impl GlobalRuntimeNetwork {
    pub fn new(
        gateway_iface: Ipv4Addr,
    ) -> std::result::Result<GlobalRuntimeNetwork, IptablesError> {
        Ok(GlobalRuntimeNetwork {
            iptables: Iptables::new(true)?,
            gateway_iface,
        })
    }
}

#[async_trait]
impl RuntimeNetwork for GlobalRuntimeNetwork {
    /// Global runtime network init will setup the whole network configuration
    /// for the system to work with workloads.
    ///
    /// This includes:
    /// - Create a RIKLET chain on the NAT table that will handle all port
    ///   redirections to workloads
    /// - Creates a rule on chain PREROUTING of table NAT that will redirect
    ///   traffic to chain RIKLET to handle port redirections
    /// - Create a rule on chain OUTPUT of table NAT that will redirect
    ///  traffic to chain RIKLET to handle port redirections
    /// - Enable MASQUERADE on the NAT table to allow workloads to access the
    ///  external network
    /// - Enable conntrack on the Filter table to allow workloads to access the
    ///  external network
    ///
    /// The usage of the RIKLET chain allows us to prevent the need to repeat
    /// the rule on both PREROUTING and OUTPUT chains.
    async fn init(&mut self) -> Result<()> {
        let chain = Chain::Custom("RIKLET".to_string());
        self.iptables
            .create_chain(&chain, &Table::Nat)
            .map_err(NetworkError::IptablesError)?;

        let nat_prerouting_redirect = Rule {
            chain: Chain::PreRouting,
            table: Table::Nat,
            rule: "-m addrtype --dst-type LOCAL -j RIKLET".to_string(),
        };

        let nat_output_redirect = Rule {
            chain: Chain::Output,
            table: Table::Nat,
            rule: "-m addrtype --dst-type LOCAL -j RIKLET".to_string(),
        };

        self.iptables
            .create(&nat_prerouting_redirect)
            .map_err(NetworkError::IptablesError)?;
        self.iptables
            .create(&nat_output_redirect)
            .map_err(NetworkError::IptablesError)?;

        let nat_masquerade = Rule {
            chain: Chain::PostRouting,
            table: Table::Nat,
            rule: format!("-o {} -j MASQUERADE", self.gateway_iface),
        };

        let filter_conntrack = Rule {
            chain: Chain::Forward,
            table: Table::Filter,
            rule: "-m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT".to_string(),
        };
        self.iptables
            .create(&nat_masquerade)
            .map_err(NetworkError::IptablesError)?;
        self.iptables
            .create(&filter_conntrack)
            .map_err(NetworkError::IptablesError)?;
        Ok(())
    }

    /// Nothing is needed to be done here, since all the iptable rules and
    /// chains are deleted from the drop implementation of iptables
    /// See [Iptables::drop]
    async fn destroy(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use crate::runtime::network::{GlobalRuntimeNetwork, RuntimeNetwork};
    use serial_test::serial;

    const GATEWAY_MOCK: Ipv4Addr = Ipv4Addr::new(192, 168, 0, 1);

    #[tokio::test]
    #[serial]
    async fn test_network_init_ok() {
        let mut network = GlobalRuntimeNetwork::new(GATEWAY_MOCK).unwrap();
        let result = network.init().await;
        assert!(result.is_ok());
        let result = network.destroy().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn test_network_init_drop() {
        let mut network = GlobalRuntimeNetwork::new(GATEWAY_MOCK).unwrap();
        let result = network.init().await;
        assert!(result.is_ok());

        let ipt = iptables::new(false).unwrap();

        let chain = ipt.chain_exists("nat", "RIKLET").unwrap();
        assert!(chain);

        let prerouting_rule = ipt
            .exists(
                "nat",
                "PREROUTING",
                "-m addrtype --dst-type LOCAL -j RIKLET",
            )
            .unwrap();
        assert!(prerouting_rule);

        let output_rule = ipt
            .exists("nat", "OUTPUT", "-m addrtype --dst-type LOCAL -j RIKLET")
            .unwrap();
        assert!(output_rule);

        drop(network);
        let ipt = iptables::new(false).unwrap();

        let chain = ipt.chain_exists("nat", "RIKLET").unwrap();
        assert!(!chain);

        let prerouting_rule = ipt
            .exists(
                "nat",
                "PREROUTING",
                "-m addrtype --dst-type LOCAL -j RIKLET",
            )
            .unwrap();
        assert!(!prerouting_rule);

        let output_rule = ipt
            .exists("nat", "OUTPUT", "-m addrtype --dst-type LOCAL -j RIKLET")
            .unwrap();
        assert!(!output_rule);
    }

    #[tokio::test]
    #[serial]
    async fn test_multiple_global_network_fails() {
        let mut network = GlobalRuntimeNetwork::new(GATEWAY_MOCK).unwrap();
        let result = network.init().await;
        assert!(result.is_ok());

        let mut network2 = GlobalRuntimeNetwork::new(GATEWAY_MOCK).unwrap();
        let result = network2.init().await;
        assert!(result.is_err());

        let result = network.destroy().await;
        assert!(result.is_ok());
    }

    fn test_to_string_gateway() {
        assert_eq!(GATEWAY_MOCK.to_string(), "192.168.0.1".to_string());
    }
}
