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
use crate::utils::init_logger;
use anyhow::Result;

use tracing::error;

#[tokio::main]
async fn main() -> Result<()> {
    init_logger(Some("debug".to_string()))?;

    // run a function to test #[instrument] macro
    // test_instrument();
    // If the process doesn't have root privileges, exit and display error.
    if !nix::unistd::Uid::effective().is_root() {
        error!("Riklet must run with root privileges.");
        std::process::exit(1);
    }

    Riklet::new()
        .await
        .unwrap_or_else(|e| {
            error!(
                "An error occured during the bootstraping process of the Riklet. {}",
                e
            );
            std::process::exit(2);
        })
        .run()
        .await?;

    Ok(())
}
