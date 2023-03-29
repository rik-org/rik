use crate::cli::resource::{CreateResource, GetMultipleResource};
use crate::cli::Handler;
use clap::Args;

/// Create a resource on the cluster.
#[derive(Debug, Args)]
pub struct CreateCommand {
    #[clap(subcommand)]
    resource: CreateResource,
}

impl CreateCommand {
    pub fn command(self) -> Box<dyn Handler> {
        match self.resource {
            CreateResource::Workloads(handler) => Box::new(handler),
            CreateResource::Instance(handler) => Box::new(handler),
        }
    }
}

#[derive(Debug, Args)]
pub struct GetMultipleCommand {
    #[clap(subcommand)]
    resource: GetMultipleResource,
}

impl GetMultipleCommand {
    pub fn command(self) -> Box<dyn Handler> {
        match self.resource {
            GetMultipleResource::Instances(handler) => Box::new(handler),
            GetMultipleResource::Workloads(handler) => Box::new(handler),
        }
    }
}
