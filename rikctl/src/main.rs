mod cli;
mod core;

use crate::cli::CommandLineInterface;
use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    CommandLineInterface::parse().command().handler().await
}
