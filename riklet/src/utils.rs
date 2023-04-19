use anyhow::Result;
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
        .with(EnvFilter::from_default_env())
        .init();
    Ok(())
}
