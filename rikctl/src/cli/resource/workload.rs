use anyhow::Result;
use async_trait::async_trait;
use clap::Args;
use prettytable::row;
use std::path::PathBuf;

use crate::cli::Handler;
use crate::core::client::{Client, WorkloadClient};
use crate::core::config::Configuration;
use crate::core::get_display_table;
use crate::core::workload::Workload;

#[derive(Debug, Args)]
pub struct CreateWorkload {
    /// Path to a JSON file that contains the workload definition.
    #[clap(short, long)]
    pub file: PathBuf,

    /// If present, the output of the command will only be the ID of the workload.
    #[clap(short, long)]
    pub quiet: bool,
}

#[async_trait]
impl Handler for CreateWorkload {
    async fn handler(&self) -> Result<()> {
        let config = Configuration::load()?;

        // Parse the workload file
        let workload = Workload::try_from(self.file.clone())?;
        let workload_id = Client::init(config.cluster)
            .create_workload(&workload)
            .await?;

        println!(
            "Workload {} has been successfully created with ID : {}",
            &workload.name, workload_id
        );
        Ok(())
    }
}

#[derive(Debug, Args)]
pub struct GetMultipleWorkload {}

#[async_trait]
impl Handler for GetMultipleWorkload {
    async fn handler(&self) -> Result<()> {
        let config = Configuration::load()?;
        let workloads = Client::init(config.cluster).get_workloads().await?;

        let mut table = get_display_table();
        table.set_titles(row!["ID", "API VERSION", "NAME", "CONTAINERS"]);
        if workloads.is_empty() {
            table.add_row(row!["", "", "", ""]);
        }
        for workload in workloads {
            table.add_row(row![
                workload.id,
                workload.value.api_version,
                workload.name,
                workload.value.spec.containers.len()
            ]);
        }

        table.printstd();
        Ok(())
    }
}
