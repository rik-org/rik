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
        RikRepository::upsert(
            &connection,
            &instance.id,
            &instance.get_full_name(),
            &serde_json::to_string(&instance).unwrap(),
            "/instance",
        )
        .map_err(|e| {
            RikError::InternalCommunicationError(format!("Could not register instance: {}", e))
        })
        .map(|_| ())
    }

    fn delete_instance(&self, instance: Instance) -> Result<(), RikError> {
        let connection = self.get_connection()?;
        RikRepository::delete(&connection, &instance.id).map_err(|e| {
            RikError::InternalCommunicationError(format!("Could not delete instance: {}", e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::fixtures::db_connection;
    use definition::workload::{Spec, WorkloadKind};
    use rstest::rstest;

    #[rstest]
    fn test_fetch_instance_function_ok(db_connection: std::sync::Arc<RikDataBase>) {
        let connection = db_connection.open().unwrap();
        connection.execute("DELETE FROM cluster", []).unwrap();

        let workload_id = "workload_id";
        let instance_id = "instance_id";
        let spec = Spec {
            containers: vec![],
            function: None,
        };

        let instance = Instance::new(
            workload_id.to_string(),
            WorkloadKind::Function,
            Some(instance_id.to_string()),
            spec,
        );

        let instance_repository = InstanceRepositoryImpl::new(db_connection);

        instance_repository.register_instance(instance).unwrap();

        let fetch_instance = instance_repository
            .fetch_instance(instance_id.to_string())
            .unwrap();

        assert_eq!(fetch_instance.id, instance_id);
    }

    #[rstest]
    fn test_fetch_instance_pod_ok(db_connection: std::sync::Arc<RikDataBase>) {
        let connection = db_connection.open().unwrap();
        connection.execute("DELETE FROM cluster", []).unwrap();

        let workload_id = "workload_id";
        let instance_id = "instance_id";
        let spec = Spec {
            containers: vec![],
            function: None,
        };

        let instance = Instance::new(
            workload_id.to_string(),
            WorkloadKind::Pod,
            Some(instance_id.to_string()),
            spec,
        );

        let instance_repository = InstanceRepositoryImpl::new(db_connection);

        instance_repository.register_instance(instance).unwrap();

        let fetch_instance = instance_repository
            .fetch_instance(instance_id.to_string())
            .unwrap();

        assert_eq!(fetch_instance.id, instance_id);
    }

    #[rstest]
    fn test_fetch_instance_not_found(db_connection: std::sync::Arc<RikDataBase>) {
        let instance_repository = InstanceRepositoryImpl::new(db_connection);
        let fetch_instance = instance_repository.fetch_instance("instance_id".to_string());
        assert!(fetch_instance.is_err());
    }

    #[rstest]
    fn test_register_instance_function_ok(db_connection: std::sync::Arc<RikDataBase>) {
        let connection = db_connection.open().unwrap();
        connection.execute("DELETE FROM cluster", []).unwrap();

        let workload_id = "workload_id";
        let instance_id = "instance_id";
        let spec = Spec {
            containers: vec![],
            function: None,
        };

        let instance = Instance::new(
            workload_id.to_string(),
            WorkloadKind::Function,
            Some(instance_id.to_string()),
            spec,
        );

        let instance_repository = InstanceRepositoryImpl::new(db_connection);

        instance_repository.register_instance(instance).unwrap();

        let fetch_instance = instance_repository
            .fetch_instance(instance_id.to_string())
            .unwrap();

        assert_eq!(fetch_instance.id, instance_id);
    }

    #[rstest]
    fn test_register_instance_pod_ok(db_connection: std::sync::Arc<RikDataBase>) {
        let connection = db_connection.open().unwrap();
        connection.execute("DELETE FROM cluster", []).unwrap();

        let workload_id = "workload_id";
        let instance_id = "instance_id";
        let spec = Spec {
            containers: vec![],
            function: None,
        };

        let instance = Instance::new(
            workload_id.to_string(),
            WorkloadKind::Pod,
            Some(instance_id.to_string()),
            spec,
        );

        let instance_repository = InstanceRepositoryImpl::new(db_connection);

        instance_repository.register_instance(instance).unwrap();

        let fetch_instance = instance_repository
            .fetch_instance(instance_id.to_string())
            .unwrap();

        assert_eq!(fetch_instance.id, instance_id);
    }
}
