use crate::api::RikError;
use crate::core::instance::Instance;
use crate::core::InstanceRepository;
use crate::database::{RikDataBase, RikRepository};
use rusqlite::Connection;
use std::sync::Arc;

pub struct InstanceRepositoryImpl {
    database: Arc<RikDataBase>,
}

impl InstanceRepositoryImpl {
    pub fn new(database: Arc<RikDataBase>) -> InstanceRepositoryImpl {
        InstanceRepositoryImpl { database }
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

impl InstanceRepository for InstanceRepositoryImpl {
    fn fetch_instance(&self, instance_id: String) -> Result<Instance, RikError> {
        let conn = self.get_connection()?;
        let element = RikRepository::check_duplicate_name(
            &conn,
            &format!("/instance/%/default/{}", &instance_id),
        )
        .map_err(|_| RikError::InvalidName(instance_id))?;

        serde_json::from_value::<Instance>(element.value).map_err(|e| {
            RikError::InternalCommunicationError(format!("Could not parse instance: {}", e))
        })
    }

    fn register_instance(&self, instance: Instance) -> Result<(), RikError> {
        let connection = self.get_connection()?;
        match RikRepository::upsert(
            &connection,
            &instance.id,
            &instance.get_full_name(),
            &serde_json::to_string(&instance).unwrap(),
            "/instance",
        ) {
            Ok(_) => Ok(()),
            Err(e) => Err(RikError::InternalCommunicationError(format!(
                "Could not register instance: {}",
                e
            ))),
        }
    }
}
