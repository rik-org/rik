pub mod command;
mod resource;

use crate::cli::command::{CreateCommand, GetMultipleCommand};
use anyhow::Result;
use async_trait::async_trait;
use clap::{Parser, Subcommand};

/// `Handler` is a trait that should be implemented for each of our resource.
///
/// It defines the contract & the input / output of a resource execution.
#[async_trait]
pub trait Handler {
    /// Executes the resource handler.
    ///
    /// Every resource should take no argument, has it is built at runtime with the arguments using Clap.
    /// Also, a resource must always return a `Result<()>`.
    async fn handler(&self) -> Result<()>;
}

/// The enumeration of our resource.
///
/// Each of our resource should be listed in this enumeration with the following format :
/// CommandName(CommandHandler)
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Request to create a resource on the cluster
    Create(CreateCommand),
    /// Fetch a resource from a cluster
    Get(GetMultipleCommand),
}

/// Command line interface to interact with a RIK Cluster
#[derive(Parser, Debug)]
#[clap(version, author, about, long_about = None)]
pub struct CommandLineInterface {
    #[clap(subcommand)]
    pub command: Command,
}

impl CommandLineInterface {
    pub fn command(self) -> Box<dyn Handler> {
        match self.command {
            Command::Create(subcommand) => subcommand.command(),
            Command::Get(subcommand) => subcommand.command(),
        }
    }
}
