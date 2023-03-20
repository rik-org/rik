use std::{net::Ipv4Addr, process::Command};

use std::fmt::Debug;
use thiserror::Error;
use tracing::{event, Level};

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Network error: {0}")]
    CommonNetworkError(String),

    #[error("IO error: {0}")]
    IoError(std::io::Error),

    #[error("Iptables error: {0}")]
    Iptables(IptablesError),
}

type Result<T> = std::result::Result<T, NetworkError>;

use crate::{
    cli::function_config::FnConfiguration,
    iptables::{rule::Rule, Chain, Iptables, IptablesError, MutateIptables, Table},
    structs::WorkloadDefinition,
    IP_ALLOCATOR,
};

pub trait RuntimeNetwork: Send + Sync + Debug {
    fn init(&self) -> Result<()>;

    fn destroy(&self) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct FunctionRuntimeNetwork {
    pub mask_long: String,
    pub firecracker_ip: Ipv4Addr,
    pub tap_ip: Ipv4Addr,
    pub function_config: FnConfiguration,
    pub default_agent_port: u16,
    pub workload_definition: WorkloadDefinition,
}

impl FunctionRuntimeNetwork {
    pub fn new(workload_definition: &WorkloadDefinition) -> Result<Self> {
        let default_agent_port: u16 = 8080;
        let mask_long: &str = "255.255.255.252";

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
            workload_definition: workload_definition.clone(),
        })
    }
}

impl RuntimeNetwork for FunctionRuntimeNetwork {
    fn init(&self) -> Result<()> {
        println!("Function network initialized");

        // Get port to expose function

        let output = Command::new("/bin/sh")
            .arg(self.function_config.script_path.clone())
            .arg(&self.workload_definition.name)
            .arg(self.tap_ip.to_string())
            .output()
            .map_err(NetworkError::IoError)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.is_empty() {
                event!(Level::ERROR, "stderr: {}", stderr);
            }
            // return Err(stderr.into());
        }

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

    fn destroy(&self) -> Result<()> {
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

impl RuntimeNetwork for PodRuntimeNetwork {
    fn init(&self) -> Result<()> {
        todo!()
    }

    fn destroy(&self) -> Result<()> {
        todo!()
    }
}
