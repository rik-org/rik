mod instance;
mod workload;

use crate::cli::resource::instance::{CreateInstance, GetMultipleInstance};
use crate::cli::resource::workload::{CreateWorkload, GetMultipleWorkload};
use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum CreateResource {
    /// Create a workload
    Workloads(CreateWorkload),
    /// Create an instance
    Instance(CreateInstance),
}

#[derive(Debug, Subcommand)]
pub enum GetMultipleResource {
    Instances(GetMultipleInstance),
    Workloads(GetMultipleWorkload),
}
