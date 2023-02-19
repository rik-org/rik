use crate::api::RikError;
use crate::core::WorkerRepository;
use crate::database::{RikDataBase, RikRepository};
use rusqlite::Connection;
use std::sync::Arc;

pub struct WorkerRepositoryImpl {
    database: Arc<RikDataBase>,
}

impl WorkerRepositoryImpl {
    pub fn new(database: Arc<RikDataBase>) -> WorkerRepositoryImpl {
        WorkerRepositoryImpl { database }
    }

    fn get_connection(&self) -> Result<Connection, RikError> {
        self.database.open().map_err(|e| {
            RikError::InternalCommunicationError(format!(
                "Could not open database connection: {}",
                e
            ))
        })
    }
}

impl WorkerRepository for WorkerRepositoryImpl {
    fn fetch_worker_address(&self, worker_id: String) -> Result<String, RikError> {
        let conn = self.get_connection()?;
        // "any" might correspond to the feature the worker can execute in the future
        // (container riklet vs dummy riklet vs function riklet)
        let element =
            RikRepository::check_duplicate_name(&conn, &format!("/worker/any/{}", &worker_id))
                .map_err(|_| RikError::InvalidName(worker_id))?;

        serde_json::from_value::<String>(element.value).map_err(|e| {
            RikError::InternalCommunicationError(format!("Could not parse worker: {}", e))
        })
    }

    fn register_worker(&self, worker_id: String, address: String) -> Result<(), RikError> {
        let connection = self.get_connection()?;
        match RikRepository::upsert(
            &connection,
            &worker_id,
            &format!("/worker/any/{}", &worker_id),
            &serde_json::to_string(&address).unwrap(),
            "/worker",
        ) {
            Ok(_) => Ok(()),
            Err(e) => Err(RikError::InternalCommunicationError(format!(
                "Could not register worker: {}",
                e
            ))),
        }
    }
}
