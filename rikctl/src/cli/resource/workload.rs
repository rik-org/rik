use anyhow::Result;
use async_trait::async_trait;
use clap::Args;
use prettytable::{row, Row};
use std::path::PathBuf;
use tracing::{debug, trace};

use crate::cli::Handler;
use crate::core::client::{Client, ResponseEntity, WorkloadClient};
use crate::core::config::Configuration;
use crate::core::workload::Workload;

use super::DisplayResource;

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
    #[tracing::instrument(name = "GetMultipleWorkload::handler", skip(self))]
    async fn handler(&self) -> Result<()> {
        let config = Configuration::load()?;
        let workloads = Client::init(config.cluster).get_workloads().await?;

        let table = workloads.into_table();
        table.printstd();
        Ok(())
    }
}

impl DisplayResource for Vec<ResponseEntity<Workload>> {
    #[tracing::instrument(name = "DisplayResource::workload::into_table", skip(self))]
    fn into_table(&self) -> prettytable::Table {
        let mut table = Self::new_table();
        table.set_titles(row!["ID", "NAME", "KIND", "CONTAINERS"]);
        if self.is_empty() {
            table.add_row(row!["", "", "", ""]);
        }
        for workload in self {
            table.add_row(row![
                workload.id,
                workload.name,
                workload.value.kind,
                workload.value.spec.containers.len()
            ]);
        }
        table
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::workload::Spec;
    use pretty_assertions::assert_eq;

    fn create_workload(name: &str) -> Workload {
        Workload {
            kind: "Workload".to_string(),
            api_version: "v1".to_string(),
            name: name.to_string(),
            spec: Spec { containers: vec![] },
        }
    }

    #[test]
    fn display_workloads_table() {
        let workloads = vec![
            ResponseEntity {
                id: "abde".to_string(),
                name: "workload-1".to_string(),
                value: create_workload("workload-1"),
            },
            ResponseEntity {
                id: "abcd".to_string(),
                name: "workload-2".to_string(),
                value: create_workload("workload-2"),
            },
        ];

        let table = workloads.into_table();
        let expected_output = r#" ID    NAME        KIND      CONTAINERS 
 abde  workload-1  Workload  0 
 abcd  workload-2  Workload  0 
"#;
        assert_eq!(table.to_string(), expected_output);
    }
}
