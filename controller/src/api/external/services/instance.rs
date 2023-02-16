use crate::api::{ApiChannel, CRUD};
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

    let instance = Instance::new(
        workload_id.clone(),
        workload.kind.clone().into(),
        name.clone(),
    );
    match RikRepository::upsert(
        connection,
        &instance.id,
        &instance.get_full_name(),
        &serde_json::to_string(&instance).unwrap(),
        "/instance",
    ) {
        Ok(_) => (),
        Err(err) => panic!("{}", err),
    }

    internal_sender
        .send(ApiChannel {
            action: CRUD::Create,
            workload_id: Some(workload_id),
            workload_definition: Some(workload),
            instance_id: Some(instance.id),
        })
        .unwrap();
}
