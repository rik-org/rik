mod cli;
mod constants;
mod core;
mod emitters;
mod iptables;
mod network;
mod runtime;
mod structs;
mod traits;
mod utils;

use crate::core::Riklet;
use anyhow::Result;
use tracing::{event, Level};
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
    // run a function to test #[instrument] macro
    // test_instrument();
    // If the process doesn't have root privileges, exit and display error.
    if !nix::unistd::Uid::effective().is_root() {
        event!(Level::ERROR, "Riklet must run with root privileges.");
        std::process::exit(1);
    }

    Riklet::new()
        .await
        .unwrap_or_else(|_| {
            event!(
                Level::ERROR,
                "An error occured during the bootstraping process of the Riklet."
            );
            std::process::exit(2);
        })
        .run()
        .await?;

    Ok(())
}
