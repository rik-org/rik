use route_recognizer;
use rusqlite::Connection;
use std::io;
use std::str::FromStr;
use std::sync::mpsc::Sender;
use tracing::{event, Level};

use crate::api;
use crate::api::external::services::element::elements_set_right_name;
use crate::api::external::services::instance::send_create_instance;
use crate::api::types::element::OnlyId;
use crate::api::types::instance::InstanceDefinition;
use crate::api::{ApiChannel, Crud};
use crate::core::instance::Instance;
use crate::database::RikRepository;

pub fn get(
    _: &mut tiny_http::Request,
    _: &route_recognizer::Params,
    connection: &Connection,
    _: &Sender<ApiChannel>,
) -> Result<tiny_http::Response<io::Cursor<Vec<u8>>>, api::RikError> {
    if let Ok(mut instances) = RikRepository::find_all(connection, "/instance") {
        instances = elements_set_right_name(instances.clone());
        let instances_json = serde_json::to_string(&instances).unwrap();
        event!(Level::INFO, "instances.get, instances found");
        Ok(tiny_http::Response::from_string(instances_json)
            .with_header(tiny_http::Header::from_str("Content-Type: application/json").unwrap())
            .with_status_code(tiny_http::StatusCode::from(200)))
    } else {
        Ok(tiny_http::Response::from_string("Cannot find instances")
            .with_status_code(tiny_http::StatusCode::from(500)))
    }
}

pub fn create(
    req: &mut tiny_http::Request,
    _: &route_recognizer::Params,
    connection: &Connection,
    internal_sender: &Sender<ApiChannel>,
) -> Result<tiny_http::Response<io::Cursor<Vec<u8>>>, api::RikError> {
    let mut content = String::new();
    req.as_reader().read_to_string(&mut content).unwrap();

    let mut instance: InstanceDefinition = serde_json::from_str(&content)?;

    //Workload not found
    if RikRepository::find_one(connection, &instance.workload_id, "/workload").is_err() {
        event!(
            Level::WARN,
            "Workload id {} not found",
            &instance.workload_id
        );
        return Ok(tiny_http::Response::from_string(format!(
            "Workload id {} not found",
            &instance.workload_id
        ))
        .with_status_code(tiny_http::StatusCode::from(404)));
    }

    if instance.name.is_some() {
        // Check name is not used
        if RikRepository::check_duplicate_name(
            connection,
            &format!("/instance/%/default/{}", instance.get_name()),
        )
        .is_ok()
        {
            event!(
                Level::WARN,
                "Instance name {} is already used",
                instance.get_name()
            );
            return Ok(tiny_http::Response::from_string("Name already used")
                .with_status_code(tiny_http::StatusCode::from(404)));
        }

        // Name cannot be used with multiple replicas
        if instance.get_replicas() > 1 {
            return Ok(
                tiny_http::Response::from_string("Cannot use name with multiple replicas")
                    .with_status_code(tiny_http::StatusCode::from(400)),
            );
        }
    }

    let mut instance_names: Vec<String> = vec![];

    for _ in 0..instance.get_replicas() {
        let instance_name = instance.name.clone().unwrap_or(Instance::generate_name());
        instance_names.push(instance_name.clone());
        send_create_instance(
            connection,
            internal_sender,
            instance.workload_id.clone(),
            &Some(instance_name),
        );
    }

    Ok(tiny_http::Response::from_string(serde_json::to_string(&instance_names).unwrap())
        .with_header(tiny_http::Header::from_str("Content-Type: application/json").unwrap())
        .with_status_code(tiny_http::StatusCode::from(201)))
}

pub fn delete(
    req: &mut tiny_http::Request,
    _: &route_recognizer::Params,
    connection: &Connection,
    internal_sender: &Sender<ApiChannel>,
) -> Result<tiny_http::Response<io::Cursor<Vec<u8>>>, api::RikError> {
    let mut content = String::new();
    req.as_reader().read_to_string(&mut content).unwrap();
    let OnlyId { id: delete_id } = serde_json::from_str(&content)?;

    if let Ok(instance) = RikRepository::find_one(connection, &delete_id, "/instance") {
        internal_sender
            .send(ApiChannel {
                action: Crud::Delete,
                workload_id: None,
                workload_definition: None,
                instance_id: Some(delete_id),
            })
            .unwrap();
        RikRepository::delete(connection, &instance.id).unwrap();

        event!(
            Level::INFO,
            "Instance {} has been requested to be deleted",
            instance.id
        );
        Ok(tiny_http::Response::from_string("").with_status_code(tiny_http::StatusCode::from(204)))
    } else {
        event!(Level::ERROR, "Instance id {} not found", delete_id);
        Ok(
            tiny_http::Response::from_string(format!("Instance id {} not found", delete_id))
                .with_status_code(tiny_http::StatusCode::from(404)),
        )
    }
}
