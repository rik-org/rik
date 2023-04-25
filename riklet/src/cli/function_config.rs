use std::path::PathBuf;

use super::CliConfiguration;
use clap::Parser;

#[derive(Debug, Clone)]
pub struct FnConfiguration {
    pub firecracker_location: PathBuf,
    pub kernel_location: PathBuf,
}

impl From<CliConfiguration> for FnConfiguration {
    fn from(cli: CliConfiguration) -> Self {
        FnConfiguration {
            firecracker_location: cli.firecracker_path,
            kernel_location: cli.kernel_path,
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
