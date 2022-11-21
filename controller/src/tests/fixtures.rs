use crate::api::ApiChannel;
use crate::database::RikDataBase;
use crate::logger::LoggingChannel;
use rstest::fixture;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};

#[fixture]
pub fn db_connection() -> std::sync::Arc<RikDataBase> {
    std::env::set_var("DATABASE_LOCATION", "/tmp/riktest");
    let db = RikDataBase::new(String::from("test"));
    db.init_tables().unwrap();
    db
}

#[fixture]
pub fn mock_logger() -> Sender<LoggingChannel> {
    let (logging_sender, _) = channel::<LoggingChannel>();
    logging_sender
}

#[fixture]
pub fn mock_internal_sender() -> Sender<ApiChannel> {
    let (internal_sender, _) = channel::<ApiChannel>();
    internal_sender
}

#[fixture]
pub fn mock_external_receiver() -> Receiver<ApiChannel> {
    let (_, external_receiver) = channel::<ApiChannel>();
    external_receiver
}

// #[fixture]
// pub fn mock_server(db_connection: Connection) {
//     let external_api = external::Server::new(
//         logging_sender.clone(),
//         internal_sender.clone(),
//         external_receiver,
//     );
//     external_api.run(db_connection);
// }
