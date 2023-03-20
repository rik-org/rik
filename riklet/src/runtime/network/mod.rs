use std::{net::Ipv4Addr, process::Command};

use async_trait::async_trait;
use proto::worker::InstanceScheduling;
use std::fmt::Debug;
use thiserror::Error;
use tracing::{debug, event, Level};

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Network error: {0}")]
    CommonNetworkError(String),

    #[error("IO error: {0}")]
    IoError(std::io::Error),

    #[error("Iptables error: {0}")]
    Iptables(IptablesError),

    #[error("Json error: {0}")]
    JsonError(serde_json::Error),
}

type Result<T> = std::result::Result<T, NetworkError>;

use crate::network::net::{Net, NetworkInterfaceConfig};
use crate::{
    cli::function_config::FnConfiguration,
    iptables::{rule::Rule, Chain, Iptables, IptablesError, MutateIptables, Table},
    structs::WorkloadDefinition,
    IP_ALLOCATOR,
};

#[async_trait]
pub trait RuntimeNetwork: Send + Sync + Debug {
    async fn init(&self) -> Result<()>;

    async fn destroy(&self) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct FunctionRuntimeNetwork {
    pub mask_long: String,
    pub firecracker_ip: Ipv4Addr,
    pub tap_ip: Ipv4Addr,
    pub function_config: FnConfiguration,
    pub default_agent_port: u16,
    pub workload_definition: WorkloadDefinition,
    pub workload: InstanceScheduling,
}

impl FunctionRuntimeNetwork {
    pub fn new(workload: &InstanceScheduling) -> Result<Self> {
        let default_agent_port: u16 = 8080;
        let mask_long: &str = "255.255.255.252";
        let workload_definition: WorkloadDefinition =
            serde_json::from_str(workload.definition.as_str()).map_err(NetworkError::JsonError)?;

        // Alocate ip range for tap interface and firecracker micro VM
        let subnet = IP_ALLOCATOR
            .lock()
            .unwrap()
            .allocate_subnet()
            .ok_or("No more internal ip available")
            .map_err(|e| NetworkError::CommonNetworkError(e.to_string()))?;

        let tap_ip = subnet
            .nth(1)
            .ok_or("Fail get tap ip")
            .map_err(|e| NetworkError::CommonNetworkError(e.to_string()))?;

        let firecracker_ip = subnet
            .nth(2)
            .ok_or("Fail to get firecracker ip")
            .map_err(|e| NetworkError::CommonNetworkError(e.to_string()))?;

        Ok(FunctionRuntimeNetwork {
            mask_long: mask_long.to_string(),
            firecracker_ip,
            function_config: FnConfiguration::load(),
            tap_ip,
            default_agent_port,
            workload: workload.clone(),
            workload_definition: workload_definition.clone(),
        })
    }
}

#[async_trait]
impl RuntimeNetwork for FunctionRuntimeNetwork {
    async fn init(&self) -> Result<()> {
        println!("Function network initialized");

        let config = NetworkInterfaceConfig::new(
            self.workload.instance_id.clone(),
            self.workload_definition.name.clone(),
            self.tap_ip,
        )
        .unwrap();
        let tap = Net::new_with_tap(config).await.unwrap(); // TODO Error;
        debug!("Waiting for the microvm to start");

        // Create a new IPTables object
        let mut ipt = Iptables::new(false).map_err(NetworkError::Iptables)?;

        // Port forward microvm on the host
        let exposed_port = self.workload_definition.get_expected_port();
        let rule = Rule {
            rule: format!(
                "-p tcp --dport {} -d {} -j DNAT --to-destination {}:{}",
                exposed_port,
                self.function_config.ifnet_ip,
                self.firecracker_ip,
                self.default_agent_port
            ),
            chain: Chain::Output,
            table: Table::Nat,
        };
        ipt.create(&rule).map_err(NetworkError::Iptables)?;

        // Allow NAT on the interface connected to the internet.
        let rule = Rule {
            rule: format!("-o {} -j MASQUERADE", self.function_config.ifnet),
            chain: Chain::PostRouting,
            table: Table::Nat,
        };
        ipt.create(&rule).map_err(NetworkError::Iptables)?;

        // Add the FORWARD rules to the filter table
        let rule = Rule {
            rule: "-m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT".to_string(),
            chain: Chain::Forward,
            table: Table::Filter,
        };
        ipt.create(&rule).map_err(NetworkError::Iptables)?;
        let rule = Rule {
            rule: format!(
                "-i rik-{}-tap -o {} -j ACCEPT",
                self.workload_definition.name, self.function_config.ifnet
            ),
            chain: Chain::Forward,
            table: Table::Filter,
        };
        ipt.create(&rule).map_err(NetworkError::Iptables)?;

        Ok(())
    }

    async fn destroy(&self) -> Result<()> {
        todo!()
    }
}

#[derive(Debug)]
pub struct PodRuntimeNetwork {}

impl PodRuntimeNetwork {
    pub fn new() -> Self {
        PodRuntimeNetwork {}
    }
}

#[async_trait]
impl RuntimeNetwork for PodRuntimeNetwork {
    async fn init(&self) -> Result<()> {
        todo!()
    }

    async fn destroy(&self) -> Result<()> {
        todo!()
    }
}
