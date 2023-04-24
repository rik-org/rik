use async_trait::async_trait;
use proto::worker::InstanceScheduling;
use std::net::Ipv4Addr;
use tracing::debug;

use crate::constants::DEFAULT_FIRECRACKER_NETWORK_MASK;
use crate::net_utils;
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
    pub tap: Option<String>,
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
                    exposed_port, self.function_config.ifnet_ip, self.guest_ip, internal_port
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
                    exposed_port, self.function_config.ifnet_ip, self.guest_ip, internal_port
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

        let iface_name = net_utils::new_tap_random_name(self.identifier.clone());
        self.tap = Some(iface_name);

        Ok(())
    }

    #[tracing::instrument(skip(self), fields(identifier = %self.identifier))]
    async fn preboot(&mut self) -> Result<()> {
        let tap_name = self.tap_name()?;
        let host_ipv4 = &self.host_ip;
        debug!("Give IP address to netid: {} -> {}", self.host_ip, tap_name);

        net_utils::set_link_ipv4(
            tap_name.clone(),
            host_ipv4.clone(),
            DEFAULT_FIRECRACKER_NETWORK_MASK,
        )
        .await
        .map_err(|e| NetworkError::InterfaceIPError(e.to_string()))?;

        net_utils::set_link_up(tap_name.clone())
            .await
            .map_err(|e| NetworkError::InterfaceIPError(e.to_string()))?;

        self.up_routing()?;
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(identifier = %self.identifier))]
    async fn destroy(&self) -> Result<()> {
        debug!("Destroy function network");
        self.down_routing()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{net::Ipv4Addr, path::PathBuf, process::Command};

    use tracing::trace;

    use crate::{
        cli::function_config::FnConfiguration,
        iptables::{rule::Rule, Chain, Iptables, MutateIptables, Table},
    };

    use super::FunctionRuntimeNetwork;

    fn open_tap_shell(iface_name: &str) -> Result<(), String> {
        let tap_output = Command::new("ip")
            .args(["tuntap", "add", iface_name, "mode", "tap"])
            .output()
            .map_err(|e| e.to_string())?;

        if !tap_output.status.success() {
            return Err(format!(
                "Tap creation failed, code {}, stderr: {}",
                tap_output.status.code().unwrap(),
                String::from_utf8(tap_output.stderr).unwrap()
            ));
        }

        trace!("Shell tap create output: {:#?}", tap_output);
        return Ok(());
    }

    fn close_tap_shell(iface_name: &str) -> Result<(), String> {
        let tap_output = Command::new("ip")
            .args(["tuntap", "del", iface_name, "mode", "tap"])
            .output()
            .map_err(|e| e.to_string())?;

        if !tap_output.status.success() {
            return Err(format!(
                "Tap creation failed, code {}, stderr: {}",
                tap_output.status.code().unwrap(),
                String::from_utf8(tap_output.stderr).unwrap()
            ));
        }

        trace!("Shell tap delete output: {:#?}", tap_output);
        return Ok(());
    }

    fn create_function_network_rt(
        tap_name: &str,
        port_mapping: &Vec<(u16, u16)>,
    ) -> FunctionRuntimeNetwork {
        let fn_config = FnConfiguration {
            ifnet: tap_name.to_string(),
            ifnet_ip: Ipv4Addr::new(10, 0, 0, 1),
            firecracker_location: PathBuf::new(),
            kernel_location: PathBuf::new(),
        };
        FunctionRuntimeNetwork {
            identifier: "test".to_string(),
            mask_long: "255.255.255.200".to_string(),
            host_ip: Ipv4Addr::new(10, 0, 0, 2),
            guest_ip: Ipv4Addr::new(10, 0, 0, 1),
            function_config: fn_config,
            port_mapping: port_mapping.clone(),
            tap: Some(tap_name.to_string()),
        }
    }

    #[tokio::test]
    async fn apply_empty_network_routing() {
        let fn_rt = create_function_network_rt("riklet008", &vec![]);
        open_tap_shell(fn_rt.tap_name().unwrap().as_str()).unwrap();
        fn_rt.up_routing().unwrap();
        fn_rt.down_routing().unwrap();
        close_tap_shell(fn_rt.tap_name().unwrap().as_str()).unwrap();
    }

    #[tokio::test]
    async fn apply_exposure_network_routing() {
        let exposed_port = vec![(8080, 8080)];
        let fn_rt = create_function_network_rt("riklet010", &exposed_port);
        open_tap_shell(fn_rt.tap_name().unwrap().as_str()).unwrap();
        fn_rt.up_routing().unwrap();

        let ipt = Iptables::new(false).unwrap();
        let mut rules: Vec<Rule> = vec![];

        // Register expected rules
        for (exposed_port, internal_port) in fn_rt.port_mapping.iter() {
            rules.push(Rule {
                rule: format!(
                    "-p tcp --dport {} -d {} -j DNAT --to-destination {}:{}",
                    exposed_port, fn_rt.function_config.ifnet_ip, fn_rt.guest_ip, internal_port
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
