use async_trait::async_trait;
use proto::worker::InstanceScheduling;
use std::net::Ipv4Addr;
use tracing::debug;

use crate::network::net::{Net, NetworkInterfaceConfig};
use crate::network::tap::close_tap_shell;
use crate::{
    cli::function_config::FnConfiguration,
    iptables::{rule::Rule, Chain, Iptables, MutateIptables, Table},
    structs::WorkloadDefinition,
};

use super::{NetworkError, Result, RuntimeNetwork, IP_ALLOCATOR};

pub struct FunctionRuntimeNetwork {
    /// Unique identifier for the function deployment
    pub identifier: String,
    /// IPv4 Mask
    /// format: 255.255.255.255
    pub mask_long: String,
    /// Guest VM IP
    pub guest_ip: Ipv4Addr,
    /// Host tap interface IP
    pub host_ip: Ipv4Addr,
    pub function_config: FnConfiguration,
    pub port_mapping: Vec<(u16, u16)>,
    pub tap: Option<Net>,
}

impl FunctionRuntimeNetwork {
    pub fn new(workload: &InstanceScheduling) -> Result<Self> {
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

        let guest_ip = subnet
            .nth(1)
            .ok_or_else(|| NetworkError::Error("Fail get tap ip".to_string()))?;

        let host_ip = subnet
            .nth(2)
            .ok_or_else(|| NetworkError::Error("Fail to get firecracker ip".to_string()))?;

        Ok(FunctionRuntimeNetwork {
            mask_long: mask_long.to_string(),
            host_ip,
            function_config: FnConfiguration::load(),
            guest_ip,
            identifier: workload.instance_id.clone(),
            port_mapping: workload_definition.get_port_mapping(),
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

    /// Insert new iptables rules to forward traffic from host to guest
    #[tracing::instrument(skip(self), fields(instance_id = %self.identifier))]
    fn up_routing(&self) -> Result<()> {
        debug!("Create iptables rules");
        let mut ipt = Iptables::new(false).map_err(NetworkError::IptablesError)?;

        for (exposed_port, internal_port) in self.port_mapping.iter() {
            let rule = Rule {
                rule: format!(
                    "-p tcp --dport {} -d {} -j DNAT --to-destination {}:{}",
                    exposed_port, self.function_config.ifnet_ip, self.host_ip, internal_port
                ),
                chain: Chain::Output,
                table: Table::Nat,
            };
            ipt.create(&rule).map_err(NetworkError::IptablesError)?;
        }

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

    /// Remove previously created iptable rules on the host
    #[tracing::instrument(skip(self), fields(instance_id = %self.identifier))]
    fn down_routing(&self) -> Result<()> {
        debug!("Delete iptables rules");
        let mut ipt = Iptables::new(false).map_err(NetworkError::IptablesError)?;

        for (exposed_port, internal_port) in self.port_mapping.iter() {
            let rule = Rule {
                rule: format!(
                    "-p tcp --dport {} -d {} -j DNAT --to-destination {}:{}",
                    exposed_port, self.function_config.ifnet_ip, self.host_ip, internal_port
                ),
                chain: Chain::Output,
                table: Table::Nat,
            };
            ipt.delete(&rule).map_err(NetworkError::IptablesError)?;
        }

        let rule = Rule {
            rule: format!(
                "-i {} -o {} -j ACCEPT",
                self.tap_name()?,
                self.function_config.ifnet
            ),
            chain: Chain::Forward,
            table: Table::Filter,
        };
        ipt.delete(&rule).map_err(NetworkError::IptablesError)?;

        Ok(())
    }
}

#[async_trait]
impl RuntimeNetwork for FunctionRuntimeNetwork {
    #[tracing::instrument(skip(self), fields(identifier = %self.identifier))]
    async fn init(&mut self) -> Result<()> {
        debug!("Init function network");

        let config =
            NetworkInterfaceConfig::new_with_random_name(self.identifier.clone(), self.guest_ip)
                .map_err(NetworkError::NetworkInterfaceError)?;

        self.tap = Some(
            Net::new_with_tap(config.clone())
                .await
                .map_err(NetworkError::NetworkInterfaceError)?,
        );
        self.up_routing()?;

        Ok(())
    }

    #[tracing::instrument(skip(self), fields(identifier = %self.identifier))]
    async fn destroy(&self) -> Result<()> {
        debug!("Destroy function network");
        self.down_routing()?;
        close_tap_shell(&self.tap_name()?).map_err(NetworkError::NetworkInterfaceError)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::network::{net::NetworkInterfaceError, tap::close_tap_shell};

    use super::*;

    async fn create_function_network_rt(
        tap_name: &str,
        port_mapping: &Vec<(u16, u16)>,
        config: NetworkInterfaceConfig,
    ) -> std::result::Result<FunctionRuntimeNetwork, NetworkInterfaceError> {
        let fn_config = FnConfiguration {
            ifnet: tap_name.to_string(),
            ifnet_ip: Ipv4Addr::new(10, 0, 0, 1),
            firecracker_location: PathBuf::new(),
            kernel_location: PathBuf::new(),
        };
        Ok(FunctionRuntimeNetwork {
            identifier: "test".to_string(),
            mask_long: "255.255.255.200".to_string(),
            host_ip: Ipv4Addr::new(10, 0, 0, 2),
            guest_ip: Ipv4Addr::new(10, 0, 0, 1),
            function_config: fn_config,
            port_mapping: port_mapping.clone(),
            tap: Some(Net::new_with_tap(config).await?),
        })
    }

    #[tokio::test]
    async fn apply_empty_network_routing() {
        let network_tap_config = NetworkInterfaceConfig::new_with_random_name(
            "riklet008".to_string(),
            Ipv4Addr::new(10, 0, 0, 223),
        )
        .unwrap();
        let fn_rt = create_function_network_rt("riklet008", &vec![], network_tap_config)
            .await
            .unwrap();
        fn_rt.up_routing().unwrap();
        fn_rt.down_routing().unwrap();
        close_tap_shell(fn_rt.tap_name().unwrap().as_str()).unwrap();
    }

    #[tokio::test]
    async fn apply_exposure_network_routing() {
        let network_tap_config = NetworkInterfaceConfig::new_with_random_name(
            "riklet008".to_string(),
            Ipv4Addr::new(10, 0, 0, 223),
        )
        .unwrap();
        let exposed_port = vec![(8080, 8080)];
        let fn_rt = create_function_network_rt("riklet008", &exposed_port, network_tap_config)
            .await
            .unwrap();
        fn_rt.up_routing().unwrap();

        let ipt = Iptables::new(false).unwrap();
        let mut rules: Vec<Rule> = vec![];

        // Register expected rules
        for (exposed_port, internal_port) in fn_rt.port_mapping.iter() {
            rules.push(Rule {
                rule: format!(
                    "-p tcp --dport {} -d {} -j DNAT --to-destination {}:{}",
                    exposed_port, fn_rt.function_config.ifnet_ip, fn_rt.host_ip, internal_port
                ),
                chain: Chain::Output,
                table: Table::Nat,
            });
            rules.push(Rule {
                rule: format!(
                    "-i {} -o {} -j ACCEPT",
                    fn_rt.tap_name().unwrap(),
                    fn_rt.function_config.ifnet
                ),
                chain: Chain::Forward,
                table: Table::Filter,
            });
        }

        // Assert they exists
        for rule in &rules {
            println!("Checking rule: {}", rule.to_string());
            assert!(ipt.exists(rule).unwrap());
        }
        fn_rt.down_routing().unwrap();

        // Assert they are deleted
        for rule in &rules {
            println!("Checking rule: {}", rule.to_string());
            assert!(!ipt.exists(rule).unwrap());
        }
        close_tap_shell(fn_rt.tap_name().unwrap().as_str()).unwrap();
    }
}
