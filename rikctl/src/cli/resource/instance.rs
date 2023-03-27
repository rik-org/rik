use crate::core::client::{Client, ResponseEntity};
use crate::core::instance::Instance;
use crate::{
    cli::Handler,
    core::{client::InstanceClient, config::Configuration},
};
use anyhow::Result;
use async_trait::async_trait;
use clap::Args;
use prettytable::row;

use super::DisplayResource;
#[derive(Debug, Args)]
pub struct CreateInstance {
    #[clap(short, long)]
    pub workload_id: String,

    #[clap(short, long)]
    pub replicas: Option<usize>,
}

#[async_trait]
impl Handler for CreateInstance {
    async fn handler(&self) -> Result<()> {
        println!("Create an instance of a workload");
        let config = Configuration::load()?;

        Client::init(config.cluster)
            .create_instance(&self.workload_id, &self.replicas)
            .await?;

        println!(
            "Instance has been successfully created for workload : {}",
            &self.workload_id
        );
        Ok(())
    }
}

#[derive(Debug, Args)]
pub struct GetMultipleInstance {}

#[async_trait]
impl Handler for GetMultipleInstance {
    async fn handler(&self) -> Result<()> {
        let config = Configuration::load()?;
        let instances = Client::init(config.cluster).get_instances().await?;

        let table = instances.into_table();

        table.printstd();
        Ok(())
    }
}

impl DisplayResource for Vec<ResponseEntity<Instance>> {
    #[tracing::instrument(name = "DisplayResource::instance::into_table", skip(self))]
    fn into_table(&self) -> prettytable::Table {
        let mut table = Self::new_table();
        table.set_titles(row!["ID", "NAME", "STATUS"]);
        if self.is_empty() {
            table.add_row(row!["", "", ""]);
        }
        for instance in self {
            table.add_row(row![instance.id, instance.name, instance.value.status]);
        }
        table
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn create_instance() -> Instance {
        Instance {
            status: "Running".to_string(),
        }
    }

    #[test]
    fn display_instances_table() {
        let instances = vec![
            ResponseEntity {
                id: "abde".to_string(),
                name: "instance-1".to_string(),
                value: create_instance(),
            },
            ResponseEntity {
                id: "abcd".to_string(),
                name: "instance-2".to_string(),
                value: create_instance(),
            },
        ];

        let table = instances.into_table();
        let expected_output = r#" ID    NAME        STATUS 
 abde  instance-1  Running 
 abcd  instance-2  Running 
"#;
        assert_eq!(table.to_string(), expected_output);
    }
}
