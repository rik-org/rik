use std::net::Ipv4Addr;

use super::CliConfiguration;
use clap::Parser;

#[derive(Debug)]
pub struct FnConfiguration {
    pub firecracker_location: String,
    pub rootfs_location: String,
    pub kernel_location: String,
    pub ifnet: String,
    pub ifnet_ip: Ipv4Addr,
    pub script_path: String,
}

impl From<CliConfiguration> for FnConfiguration {
    fn from(cli: CliConfiguration) -> Self {
        FnConfiguration {
            firecracker_location: cli.firecracker_path,
            rootfs_location: cli.rootfs_path,
            kernel_location: cli.kernel_path,
            ifnet: cli.ifnet,
            ifnet_ip: cli.ifnet_ip,
            script_path: cli.script_path,
        }
    }
}

impl FnConfiguration {
    fn get_cli_args() -> CliConfiguration {
        CliConfiguration::parse()
    }

    pub fn load() -> Self {
        let opts = FnConfiguration::get_cli_args();
        FnConfiguration::from(opts)
    }
}
