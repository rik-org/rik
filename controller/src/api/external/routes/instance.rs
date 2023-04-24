use definition::workload::WorkloadDefinition;
use route_recognizer;
use rusqlite::Connection;
use std::str::FromStr;
use std::sync::mpsc::Sender;
use tracing::{event, Level};

use crate::api::external::services::element::elements_set_right_name;
use crate::api::external::services::instance::send_create_instance;
use crate::api::types::element::OnlyId;
use crate::api::types::instance::InstanceDefinition;
use crate::api::{ApiChannel, Crud, RikError};
use crate::core::instance::Instance;
use crate::database::RikRepository;

use super::HttpResult;

pub fn get(
    _: &mut tiny_http::Request,
    _: &route_recognizer::Params,
    connection: &Connection,
    _: &Sender<ApiChannel>,
) -> HttpResult {
    if let Ok(mut instances) = RikRepository::find_all(connection, "/instance") {
        instances = elements_set_right_name(instances.clone());
        let instances_json = serde_json::to_string(&instances).map_err(RikError::ParsingError)?;

        event!(Level::INFO, "instances.get, instances found");

        Ok(tiny_http::Response::from_string(instances_json)
            .with_header(
                tiny_http::Header::from_str("Content-Type: application/json").map_err(|_| {
                    RikError::Error("tiny_http::Header failed to add header".to_string())
                })?,
            )
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
) -> HttpResult {
    let mut content = String::new();
    req.as_reader()
        .read_to_string(&mut content)
        .map_err(RikError::IoError)?;

    let mut instance: InstanceDefinition =
        serde_json::from_str(&content).map_err(RikError::ParsingError)?;

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

    Ok(tiny_http::Response::from_string(
        serde_json::to_string(&instance_names).map_err(RikError::ParsingError)?,
    )
    .with_header(
        tiny_http::Header::from_str("Content-Type: application/json")
            .map_err(|_| RikError::Error("tiny_http::Header failed to add header".to_string()))?,
    )
    .with_status_code(tiny_http::StatusCode::from(201)))
}

pub fn delete(
    req: &mut tiny_http::Request,
    _: &route_recognizer::Params,
    connection: &Connection,
    internal_sender: &Sender<ApiChannel>,
) -> HttpResult {
    let mut content = String::new();
    req.as_reader()
        .read_to_string(&mut content)
        .map_err(RikError::IoError)?;
    let OnlyId { id: delete_id } =
        serde_json::from_str(&content).map_err(RikError::ParsingError)?;

    if let Ok(instance) = RikRepository::find_one(connection, &delete_id, "/instance") {
        let instance_def: InstanceDefinition =
            serde_json::from_value(instance.value.clone()).map_err(RikError::ParsingError)?;

        let workload_def_rs =
            RikRepository::find_one(connection, &instance_def.workload_id, "/workload");
        if let Err(e) = workload_def_rs {
            event!(
                Level::ERROR,
                "Could not find workload id {} while should have been able to, error: {}",
                instance_def.workload_id,
                e
            );
            return Ok(tiny_http::Response::from_string(format!(
                "Workload {} matching the instance ID is not found",
                instance_def.workload_id
            ))
            .with_status_code(tiny_http::StatusCode::from(404)));
        }
        let workload_def: WorkloadDefinition =
            serde_json::from_value(workload_def_rs.map_err(RikError::DataBaseError)?.value)
                .map_err(RikError::ParsingError)?;
        internal_sender
            .send(ApiChannel {
                action: Crud::Delete,
                workload_id: Some(instance_def.workload_id),
                workload_definition: Some(workload_def),
                instance_id: Some(delete_id),
            })
            .map_err(|e| RikError::InternalCommunicationError(e.to_string()))?;

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
