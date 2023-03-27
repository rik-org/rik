mod instance;
mod workload;

use crate::cli::resource::instance::{CreateInstance, GetMultipleInstance};
use crate::cli::resource::workload::{CreateWorkload, GetMultipleWorkload};
use clap::Subcommand;
use prettytable::{format, Table};

#[derive(Debug, Subcommand)]
pub enum CreateResource {
    /// Create a workload
    Workload(CreateWorkload),
    /// Create an instance
    Instance(CreateInstance),
}

#[derive(Debug, Subcommand)]
pub enum GetMultipleResource {
    /// List instances
    Instances(GetMultipleInstance),
    /// List workloads,
    Workload(GetMultipleWorkload),
}

/// Trait which defines how resources should be displayed
trait DisplayResource<T = Self>
where
    T: Sized,
{
    fn new_table() -> Table {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_CLEAN);
        table
    }
    /// Prints the list of resources in form of table
    fn into_table(&self) -> Table;
}
