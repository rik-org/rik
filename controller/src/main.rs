mod api;
mod core;
mod database;
mod tests;

use std::sync::mpsc::channel;
use std::thread;

use crate::{api::RikError, database::RikDataBase};
use api::{external, ApiChannel};
use tracing::{error, event, metadata::LevelFilter, Level};
use tracing_subscriber::{
    fmt, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

use crate::core::core::Core;
use tokio::runtime::Builder;

fn logger_setup() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();
}

#[tokio::main]
async fn main() {
    logger_setup();
    event!(Level::INFO, "Starting Rik");
    let db = RikDataBase::new(String::from("rik"));
    if let Err(e) = db.init_tables() {
        error!("Error while table initialization {}", e)
    }

    let (legacy_sender, legacy_receiver) = channel::<ApiChannel>();

    let internal_api = Core::new(db.clone())
        .await
        .expect("Failed to create internal API");
    let external_api = external::Server::new(legacy_sender);
    let mut threads = Vec::new();

    threads.push(thread::spawn(move || -> Result<(), RikError> {
        let future = async move { internal_api.listen_notification(legacy_receiver).await };
        Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(future);
        Ok(())
    }));

    threads.push(thread::spawn(move || -> Result<(), RikError> {
        external_api.run(db)
    }));

    for thread in threads {
        if let Err(e) = thread
            .join()
            .expect("Couldn't join on the associated thread")
        {
            error!("An error occured {}", e)
        }
    }
}
