mod api;
mod core;
mod database;
mod tests;

use std::sync::mpsc::channel;
use std::thread;

use crate::database::RikDataBase;
use api::{external, ApiChannel};
use tracing::{event, Level};

use crate::core::core::Core;
use tokio::runtime::Builder;

fn logger_setup() {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        // .with_file(true)
        // .with_line_number(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to initiate the logger subscriber");
}

#[tokio::main]
async fn main() {
    logger_setup();
    event!(Level::INFO, "Starting Rik");
    let db = RikDataBase::new(String::from("rik"));
    db.init_tables().unwrap();

    let (legacy_sender, legacy_receiver) = channel::<ApiChannel>();

    let internal_api = Core::new(db.clone())
        .await
        .expect("Failed to create internal API");
    let external_api = external::Server::new(legacy_sender);
    let mut threads = Vec::new();

    threads.push(thread::spawn(move || {
        let future = async move { internal_api.listen_notification(legacy_receiver).await };
        Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(future)
    }));

    threads.push(thread::spawn(move || external_api.run(db)));

    for thread in threads {
        thread.join().unwrap();
    }
}
