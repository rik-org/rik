mod cli;
mod constants;
mod core;
mod emitters;
mod iptables;
mod net_utils;
mod runtime;
mod structs;

use crate::core::Riklet;
use anyhow::Result;

use tracing::{error, metadata::LevelFilter};
use tracing_subscriber::{
    fmt, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

pub fn banner() {
    println!(
        r#"
    ______ _____ _   __ _      _____ _____
    | ___ \_   _| | / /| |    |  ___|_   _|
    | |_/ / | | | |/ / | |    | |__   | |
    |    /  | | |    \ | |    |  __|  | |
    | |\ \ _| |_| |\  \| |____| |___  | |
    \_| \_|\___/\_| \_/\_____/\____/  \_/
    "#
    );
}

pub fn init_logger() -> Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();
    Ok(())
}
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    init_logger()?;

    // If the process doesn't have root privileges, exit and display error.
    if !nix::unistd::Uid::effective().is_root() {
        error!("Riklet must run with root privileges.");
        std::process::exit(1);
    }

    let mut riklet = Riklet::new().await.unwrap_or_else(|e| {
        error!(
            "An error occured during the bootstraping process of the Riklet. {}",
            e
        );
        std::process::exit(2);
    });

    tokio::select! {
        _ = riklet.run() => {},
        _ = signal::ctrl_c() => {}
    }

    riklet.shutdown().await?;

    Ok(())
}
