mod cli;
mod core;

use crate::cli::CommandLineInterface;
use anyhow::Result;
use clap::Parser;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy()
                .add_directive("h2=OFF".parse().unwrap()), // disable all events from the `h2` crate
        )
        .init();
    CommandLineInterface::parse().command().handler().await
}
