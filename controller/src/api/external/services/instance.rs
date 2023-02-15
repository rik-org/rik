use crate::api::{ApiChannel, CRUD};
use crate::database::RikRepository;
use crate::instance::Instance;
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
    match RikRepository::insert(
        connection,
        instance.get_full_name().as_str(),
        serde_json::to_string(&instance).unwrap().as_str(),
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
