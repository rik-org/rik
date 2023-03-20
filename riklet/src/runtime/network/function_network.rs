use async_trait::async_trait;
use proto::worker::InstanceScheduling;
use std::fmt::Debug;
use std::net::Ipv4Addr;
use tracing::debug;

use crate::network::net::{Net, NetworkInterfaceConfig};
use crate::{
    cli::function_config::FnConfiguration,
    iptables::{rule::Rule, Chain, Iptables, MutateIptables, Table},
    structs::WorkloadDefinition,
};

use super::{NetworkError, Result, RuntimeNetwork, IP_ALLOCATOR};

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
        })
    }
}

#[async_trait]
impl RuntimeNetwork for FunctionRuntimeNetwork {
    async fn init(&self) -> Result<()> {
        println!("Function network initialized");

        // Port forward microvm on the host
        let exposed_port = self.workload_definition.get_expected_port();

        let config = NetworkInterfaceConfig::new(
            self.workload.instance_id.clone(),
            self.workload_definition.name.clone(),
            self.tap_ip,
        )
        .map_err(NetworkError::NetworkInterfaceError)?;

        let _tap = Net::new_with_tap(config)
            .await
            .map_err(NetworkError::NetworkInterfaceError)?;
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
        let rule = Rule {
            rule: format!(
                "-i rik-{}-tap -o {} -j ACCEPT",
                self.workload_definition.name, self.function_config.ifnet
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
