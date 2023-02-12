mod api;
mod database;
mod instance;
mod logger;
mod tests;

use std::sync::mpsc::channel;
use std::thread;

use crate::database::RikDataBase;
use api::{external, internal, ApiChannel};
use logger::{Logger, LoggingChannel};

use tokio::runtime::Builder;

fn main() {
    let db = RikDataBase::new(String::from("rik"));
    db.init_tables().unwrap();

    let (logging_sender, logging_receiver) = channel::<LoggingChannel>();
    let (internal_sender, internal_receiver) = channel::<ApiChannel>();
    let (external_sender, external_receiver) = channel::<ApiChannel>();

    let logger = Logger::new(logging_receiver, String::from("Main"));

    let logging_sender_clone = logging_sender.clone();
    let internal_api = internal::Server::new(logging_sender, external_sender, internal_receiver);
    let external_api =
        external::Server::new(logging_sender_clone, internal_sender, external_receiver);
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

    threads.push(thread::spawn(move || {
        logger.run();
    }));

    for thread in threads {
        thread.join().unwrap();
    }
}
