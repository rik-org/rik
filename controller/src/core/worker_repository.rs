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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::fixtures::db_connection;
    use rstest::rstest;

    #[rstest]
    fn test_fetch_worker_address_ok(db_connection: std::sync::Arc<RikDataBase>) {
        let connection = db_connection.open().unwrap();
        connection.execute("DELETE FROM cluster", []).unwrap();
        let worker_id = "test-worker";
        let address = "http://localhost:8080";
        let worker_repository = WorkerRepositoryImpl::new(db_connection);
        worker_repository
            .register_worker(worker_id.to_string(), address.to_string())
            .unwrap();

        let fetched_address = worker_repository
            .fetch_worker_address(worker_id.to_string())
            .unwrap();
        assert_eq!(fetched_address, address);
    }

    #[rstest]
    fn test_fetch_worker_address_not_found(db_connection: std::sync::Arc<RikDataBase>) {
        let worker_repository = WorkerRepositoryImpl::new(db_connection);
        let result = worker_repository.fetch_worker_address("test-worker".to_string());
        assert!(result.is_err());
    }

    #[rstest]
    fn test_register_worker_ok(db_connection: std::sync::Arc<RikDataBase>) {
        let worker_repository = WorkerRepositoryImpl::new(db_connection);
        let worker_id = "test-worker";
        let address = "http://localhost:8080";
        let result = worker_repository.register_worker(worker_id.to_string(), address.to_string());
        assert!(result.is_ok());
    }

    #[rstest]
    fn test_update_worker_addr(db_connection: std::sync::Arc<RikDataBase>) {
        let worker_repository = WorkerRepositoryImpl::new(db_connection);
        let worker_id = "test-worker";
        let address = "http://localhost:8080";
        worker_repository
            .register_worker(worker_id.to_string(), address.to_string())
            .unwrap();

        let new_address = "http://localhost:8081";
        worker_repository
            .register_worker(worker_id.to_string(), new_address.to_string())
            .unwrap();

        let fetched_address = worker_repository
            .fetch_worker_address(worker_id.to_string())
            .unwrap();
        assert_eq!(fetched_address, new_address);
    }
}
