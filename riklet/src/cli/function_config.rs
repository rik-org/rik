use std::{net::Ipv4Addr, path::PathBuf};

use crate::net_utils::{get_default_gateway, get_default_iface};

use super::CliConfiguration;
use clap::Parser;

#[derive(Debug, Clone)]
pub struct FnConfiguration {
    pub kernel_location: PathBuf,
    /// IP used to access the external network
    pub gateway_ip: Ipv4Addr,
    /// Network interface used to access the external network
    pub iface: String,
}

impl FnConfiguration {
    fn get_cli_args() -> CliConfiguration {
        CliConfiguration::parse()
    }

    pub fn load() -> Result<FnConfiguration, anyhow::Error> {
        let opts = FnConfiguration::get_cli_args();

        let gateway_ip = match opts.iface_ip {
            Some(ip) => ip,
            None => get_default_gateway()?,
        };
        let iface = match opts.iface {
            Some(iface) => iface,
            None => get_default_iface()?,
        };
        Ok(FnConfiguration {
            firecracker_location: opts.firecracker_path,
            kernel_location: opts.kernel_path,
            gateway_ip,
            iface,
        })
    }
}
