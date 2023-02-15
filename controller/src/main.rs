mod api;
mod database;
mod instance;
mod tests;

use std::sync::mpsc::channel;
use std::thread;

use crate::database::RikDataBase;
use api::{external, internal, ApiChannel};
use tracing::{event, Level};

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

fn main() {
    logger_setup();
    event!(Level::INFO, "Starting Rik");
    event!(Level::INFO, "Starting Rik");
    let db = RikDataBase::new(String::from("rik"));
    db.init_tables().unwrap();

    let (internal_sender, internal_receiver) = channel::<ApiChannel>();
    let (external_sender, external_receiver) = channel::<ApiChannel>();

    let internal_api = internal::Server::new(external_sender, internal_receiver);
    let external_api = external::Server::new(internal_sender, external_receiver);
    let mut threads = Vec::new();

    let db_clone_internal = db.clone();
    threads.push(thread::spawn(move || {
        let future = async move { internal_api.run(db_clone_internal).await };
        Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(future)
    }));

    threads.push(thread::spawn(move || {
        external_api.run(db);
    }));

    for thread in threads {
        thread.join().unwrap();
    }
}
