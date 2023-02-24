pub mod config;
pub mod function_config;

use std::net::Ipv4Addr;

use clap::{value_parser, Parser};

/// The configuration of the riklet.
#[derive(Debug, Clone, Parser)]
#[command(name = "Riklet", version, about)]
pub struct CliConfiguration {
    /// The path to the Riklet configuration file. If the file not exists, it will be created.
    #[arg(
        short,
        long,
        default_value = "/etc/riklet/configuration.toml",
        help = ""
    )]
    pub config_file: String,
    /// The IP of the Rik master node.
    #[arg(short, long)]
    pub master_ip: Option<String>,
    /// The level of verbosity.
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
    /// If set and there is a config file, values defined by the CLI will override values of the configuration file.
    #[arg(long)]
    pub override_config: bool,
    /// Path to the firecarcker binary.
    #[arg(
        long,
        value_name = "FIRECRACKER_LOCATION",
        env = "FIRECRACKER_LOCATION",
        default_value = "/app/fireckracker"
    )]
    pub firecracker_path: String,
    /// Path to the rootfs.
    #[arg(
        long,
        value_name = "ROOTFS_LOCATION",
        env = "ROOTFS_LOCATION",
        default_value = "/app/rootfs.ext4"
    )]
    pub rootfs_path: String,
    /// Path to the linux kernel.
    #[arg(
        long,
        value_name = "KERNEL_LOCATION",
        env = "KERNEL_LOCATION",
        default_value = "/app/vmlinux.bin"
    )]
    pub kernel_path: String,
    /// Network interface connected to internet.
    #[arg(long, value_name = "IFNET", env = "IFNET", default_value = "eth0")]
    pub ifnet: String,
    /// IP of the network interface
    #[arg(
        long,
        value_name = "IFNET_IP",
        env = "IFNET_IP",
        value_parser = value_parser!(Ipv4Addr)
    )]
    pub ifnet_ip: Ipv4Addr,
    /// Path to the script to create tap interfaces
    #[arg(long, value_name = "SCRIPT_LOCATION", env = "SCRIPT_LOCATION")]
    pub script_path: String,
}

impl CliConfiguration {
    /// Get the log level
    pub fn get_log_level(&self) -> &str {
        match self.verbose {
            0 => "info",
            1 => "debug",
            _ => "trace",
        }
    }
}
