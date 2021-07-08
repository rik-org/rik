use crate::api::{ApiChannel, CRUD};
use crate::database::RikRepository;
use definition::workload::WorkloadDefinition;
use names::Generator;
use rusqlite::Connection;
use std::sync::mpsc::Sender;

pub fn send_create_instance(
    connection: &Connection,
    internal_sender: &Sender<ApiChannel>,
    workload_id: String,
    name: &Option<String>,
) {
    let mut random_name_generator = Generator::default();
    let random_name = random_name_generator.next().unwrap();
    let instance_name = match name {
        Some(name) => name,
        None => &random_name,
    };

    let workload_db = match RikRepository::find_one(connection, &workload_id, "/workload") {
        Ok(workload) => workload,
        Err(err) => panic!("{}", err),
    };
    let workload: WorkloadDefinition =
        serde_json::from_str(&workload_db.value.to_string()).unwrap();

    internal_sender
        .send(ApiChannel {
            action: CRUD::Create,
            workload_id: Some(workload_id),
            workload_definition: Some(workload),
            instance_id: None,
        })
        .unwrap();
}
