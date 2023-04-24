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

use tokio::signal::ctrl_c;
use tokio::signal::unix::{signal, SignalKind};
use tracing::{error, info, metadata::LevelFilter};
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

    // An infinite stream of hangup signals.
    let mut signals = signal(SignalKind::terminate())?;

    tokio::select! {
        _ = riklet.run() => {},
        _ = ctrl_c() => {
            info!("Receive SIGINT signal.");
        },
        _ = signals.recv() => {
            info!("Receive SIGTERM signal.");
        }
    }

    riklet.shutdown().await?;

    info!("Riklet stoped");

    Ok(())
}
