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
use once_cell::sync::Lazy;
use shared::utils::ip_allocator::IpAllocator;
use std::sync::Mutex;
use tracing::{event, Level};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Registry};

// Initialize Singleton for IpAllocator
static IP_ALLOCATOR: Lazy<Mutex<IpAllocator>> = Lazy::new(|| {
    let ip_allocator = IpAllocator::new().expect("Fail to load IP allocator");
    Mutex::new(ip_allocator)
});

pub fn init_logger(log_level: Option<String>) -> Result<()> {
    let logger = tracing_subscriber::fmt::layer().json();
    // Try to get the log level from the environment variable `RUST_LOG`.
    // If the variable is not defined, then use the cli argument or the default value 'info' if neither is defined
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| {
            let level = log_level.unwrap_or("info".to_string());
            EnvFilter::try_new(level.as_str())
        })?
        .add_directive("h2=OFF".parse().unwrap()); // disable all events from the `h2` crate;

    let collector = Registry::default().with(logger).with(env_filter);
    tracing::subscriber::set_global_default(collector)?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger(Some("debug".to_string()))?;

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
