use anyhow::Result;

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
    let subscriber = tracing_subscriber::fmt().compact().finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to initiate the logger subscriber");
    Ok(())
}
