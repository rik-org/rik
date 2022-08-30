use crate::core::client::Client;
use crate::{
    cli::Handler,
    core::{client::InstanceClient, config::Configuration, get_display_table},
};
use anyhow::Result;
use async_trait::async_trait;
use clap::Args;
use prettytable::row;
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

        let mut table = get_display_table();
        table.set_titles(row!["ID", "NAME", "STATUS"]);
        if instances.is_empty() {
            table.add_row(row!["", "", ""]);
        }
        for instance in instances {
            table.add_row(row![instance.id, instance.name, instance.value.status]);
        }

        table.printstd();
        Ok(())
    }
}
