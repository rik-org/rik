use std::{net::Ipv4Addr, path::PathBuf};

use super::CliConfiguration;
use clap::Parser;

#[derive(Debug, Clone)]
pub struct FnConfiguration {
    pub firecracker_location: PathBuf,
    pub kernel_location: PathBuf,
    pub ifnet: String,
    pub ifnet_ip: Ipv4Addr,
}

fn get_default_ip() -> Ipv4Addr {
    // ip r | grep default | cut -d' ' -f 3 | head -n 1
    let output = std::process::Command::new("ip")
        .arg("r")
        .arg("show")
        .arg("default")
        .output()
        .expect("Failed to execute command");
    let output = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    let output = output.split_whitespace().nth(2).unwrap();
    output.parse().unwrap()
}

fn get_default_interface() -> String {
    // ip r | grep default | cut -d' ' -f 5 | head -n 1
    let output = std::process::Command::new("ip")
        .arg("r")
        .arg("show")
        .arg("default")
        .output()
        .expect("Failed to execute command");
    let output = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    let output = output.split_whitespace().nth(4).unwrap();
    output.to_string()
}

impl From<CliConfiguration> for FnConfiguration {
    fn from(cli: CliConfiguration) -> Self {
        let ifnet = cli.ifnet.unwrap_or_else(|| get_default_interface());
        let ifnet_ip = cli.ifnet_ip.unwrap_or_else(|| get_default_ip());
        FnConfiguration {
            firecracker_location: cli.firecracker_path,
            kernel_location: cli.kernel_path,
            ifnet: ifnet,
            ifnet_ip: ifnet_ip,
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
