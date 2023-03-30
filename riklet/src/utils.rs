use anyhow::Result;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Registry};

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

pub fn init_logger(log_level: Option<String>) -> Result<()> {
    let logger = tracing_subscriber::fmt::layer().json();
    // Try to get the log level from the environment variable `RUST_LOG`.
    // If the variable is not defined, then use the cli argument or the default value 'info' if neither is defined
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| {
            let level = log_level.unwrap_or_else(|| "info".to_string());
            EnvFilter::try_new(level.as_str())
        })?
        .add_directive("h2=OFF".parse().unwrap()); // disable all events from the `h2` crate;

    let collector = Registry::default().with(logger).with(env_filter);
    tracing::subscriber::set_global_default(collector)?;

    Ok(())
}
