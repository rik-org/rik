use crate::api::{ApiChannel, Crud};
use crate::core::instance::Instance;
use crate::database::RikRepository;
use definition::workload::WorkloadDefinition;
use rusqlite::Connection;
use std::sync::mpsc::Sender;

pub fn send_create_instance(
    connection: &Connection,
    internal_sender: &Sender<ApiChannel>,
    workload_id: String,
    name: &Option<String>,
) {
    let workload_db = match RikRepository::find_one(connection, &workload_id, "/workload") {
        Ok(workload) => workload,
        Err(err) => panic!("{}", err),
    };
    let workload: WorkloadDefinition =
        serde_json::from_str(&workload_db.value.to_string()).unwrap();
    let instance_name = name.clone().unwrap_or(Instance::generate_name());

    internal_sender
        .send(ApiChannel {
            action: Crud::Create,
            workload_id: Some(workload_id),
            workload_definition: Some(workload),
            instance_id: Some(instance_name),
        })
        .unwrap();
}
