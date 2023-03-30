use async_trait::async_trait;
use proto::worker::InstanceScheduling;
use std::net::Ipv4Addr;
use tracing::{debug, info};

use crate::network::net::{Net, NetworkInterfaceConfig};
use crate::{
    cli::function_config::FnConfiguration,
    iptables::{rule::Rule, Chain, Iptables, MutateIptables, Table},
    structs::WorkloadDefinition,
};

use super::{NetworkError, Result, RuntimeNetwork, IP_ALLOCATOR};

pub struct FunctionRuntimeNetwork {
    pub mask_long: String,
    pub firecracker_ip: Ipv4Addr,
    pub tap_ip: Ipv4Addr,
    pub function_config: FnConfiguration,
    pub default_agent_port: u16,
    pub workload_definition: WorkloadDefinition,
    pub workload: InstanceScheduling,
    pub tap: Option<Net>,
}

impl FunctionRuntimeNetwork {
    pub fn new(workload: &InstanceScheduling) -> Result<Self> {
        let default_agent_port: u16 = 8080;
        let mask_long: &str = "255.255.255.252";

        let workload_definition: WorkloadDefinition =
            serde_json::from_str(workload.definition.as_str())
                .map_err(NetworkError::ParsingError)?;

        // Alocate ip range for tap interface and firecracker micro VM
        let subnet = IP_ALLOCATOR
            .lock()
            .unwrap()
            .allocate_subnet()
            .ok_or_else(|| NetworkError::Error("No more internal ip available".to_string()))?;

        let tap_ip = subnet
            .nth(1)
            .ok_or_else(|| NetworkError::Error("Fail get tap ip".to_string()))?;

        let firecracker_ip = subnet
            .nth(2)
            .ok_or_else(|| NetworkError::Error("Fail to get firecracker ip".to_string()))?;

        Ok(FunctionRuntimeNetwork {
            mask_long: mask_long.to_string(),
            firecracker_ip,
            function_config: FnConfiguration::load(),
            tap_ip,
            default_agent_port,
            workload: workload.clone(),
            workload_definition,
            tap: None,
        })
    }

    pub fn tap_name(&self) -> Result<String> {
        self.tap
            .as_ref()
            .map(|v| v.iface_name())
            .as_ref()
            .cloned()
            .ok_or_else(|| NetworkError::Error("Tap interface name not found".to_string()))
    }
}

#[async_trait]
impl RuntimeNetwork for FunctionRuntimeNetwork {
    async fn init(&mut self) -> Result<()> {
        info!("Function network initialized");

        // Port forward microvm on the host
        let exposed_port = self
            .workload_definition
            .get_expected_port()
            .ok_or_else(|| NetworkError::Error("Exposed port not found".to_string()))?;

        let config = NetworkInterfaceConfig::new_with_random_name(
            self.workload.instance_id.clone(),
            self.tap_ip,
        )
        .map_err(NetworkError::NetworkInterfaceError)?;

        self.tap = Some(
            Net::new_with_tap(config.clone())
                .await
                .map_err(NetworkError::NetworkInterfaceError)?,
        );
        debug!("Waiting for the microvm to start");

        // Create a new IPTables object
        let mut ipt = Iptables::new(false).map_err(NetworkError::IptablesError)?;

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
        ipt.create(&rule).map_err(NetworkError::IptablesError)?;

        let rule = Rule {
            rule: format!(
                "-i {} -o {} -j ACCEPT",
                self.tap_name()?,
                self.function_config.ifnet
            ),
            chain: Chain::Forward,
            table: Table::Filter,
        };
        ipt.create(&rule).map_err(NetworkError::IptablesError)?;

        Ok(())
    }

    async fn destroy(&self) -> Result<()> {
        todo!()
    }
}
