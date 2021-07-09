mod api;
mod database;
mod logger;
mod tests;

use std::sync::mpsc::channel;
use std::thread;

use crate::database::RikDataBase;
use api::{external, internal, ApiChannel};
use logger::{Logger, LoggingChannel};

use tokio::runtime::Builder;

fn main() {
    if !nix::unistd::Uid::effective().is_root() {
        println!("Rik controller must run with root privileges.");
        std::process::exit(1);
    }
    let db = RikDataBase::new(String::from("rik"));
    db.init_tables().unwrap();

    let (logging_sender, logging_receiver) = channel::<LoggingChannel>();
    let (internal_sender, internal_receiver) = channel::<ApiChannel>();
    let (external_sender, external_receiver) = channel::<ApiChannel>();

    let logger = Logger::new(logging_receiver, String::from("Main"));

    let internal_api = internal::Server::new(
        logging_sender.clone(),
        external_sender.clone(),
        internal_receiver,
    );
    let external_api = external::Server::new(
        logging_sender.clone(),
        internal_sender.clone(),
        external_receiver,
    );
    let mut threads = Vec::new();

    let db_clone_internal = db.clone();
    threads.push(thread::spawn(move || {
        let future = async move {
            let res = internal_api.run(db_clone_internal).await;
            res
        };
        let res = Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(future);
        res
    }));

    let db_clone_external = db.clone();
    threads.push(thread::spawn(move || {
        external_api.run(db_clone_external);
    }));

    threads.push(thread::spawn(move || {
        logger.run();
    }));

    for thread in threads {
        thread.join().unwrap();
    }
}
