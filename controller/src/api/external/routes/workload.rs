use crate::api::external::services::element::elements_set_right_name;
use crate::api::types::element::OnlyId;
use crate::api::{ApiChannel, CRUD};
use crate::database::RikRepository;

use crate::instance::Instance;
use anyhow::Result;
use definition::workload::WorkloadDefinition;
use log::{error, info, warn};
use route_recognizer;
use rusqlite::Connection;
use serde_json::json;
use std::io;
use std::str::FromStr;
use std::sync::mpsc::Sender;
use tiny_http::Response;
use tiny_http::Response;

type HttpResult<T = io::Cursor<Vec<u8>>> = Result<Response<T>, api::RikError>;

pub fn get(
    _: &mut tiny_http::Request,
    _: &route_recognizer::Params,
    connection: &Connection,
    _: &Sender<ApiChannel>,
) -> Result<Response<io::Cursor<Vec<u8>>>> {
    if let Ok(mut workloads) = RikRepository::find_all(connection, "/workload") {
        workloads = elements_set_right_name(workloads.clone());
        let workloads_json = serde_json::to_string(&workloads).unwrap();
        info!("Workloads found");

        Ok(tiny_http::Response::from_string(workloads_json)
            .with_header(tiny_http::Header::from_str("Content-Type: application/json").unwrap())
            .with_status_code(tiny_http::StatusCode::from(200)))
    } else {
        Ok(tiny_http::Response::from_string("Cannot find workloads")
            .with_status_code(tiny_http::StatusCode::from(500)))
    }
}

pub fn get_instances(
    _: &mut tiny_http::Request,
    params: &route_recognizer::Params,
    connection: &Connection,
    _: &Sender<ApiChannel>,
) -> HttpResult {
    let workload_id = params.find("workloadid").unwrap_or_default();

    if workload_id.is_empty() {
        return Ok(tiny_http::Response::from_string("No workload id provided")
            .with_status_code(tiny_http::StatusCode::from(400)));
    }

    // That's dirty and we know it, however it's the easiest way to do for now.
    if let Ok(elements) = RikRepository::find_all(connection, "/instance") {
        let mut instances: Vec<Instance> = elements
            .iter()
            .map(|e| serde_json::from_value(e.clone().value).unwrap())
            .filter(|instance: &Instance| instance.workload_id == workload_id)
            .collect();

        if instances.is_empty() {
            return Ok(tiny_http::Response::from_string("")
                .with_status_code(tiny_http::StatusCode::from(204)));
        }

        let instances_json = json!({ "instances": instances }).to_string();

        return Ok(tiny_http::Response::from_string(instances_json)
            .with_header(tiny_http::Header::from_str("Content-Type: application/json").unwrap())
            .with_status_code(tiny_http::StatusCode::from(200)));
    }

    Ok(
        tiny_http::Response::from_string("Could not find workload instances")
            .with_status_code(tiny_http::StatusCode::from(404)),
    )
}

pub fn create(
    req: &mut tiny_http::Request,
    _: &route_recognizer::Params,
    connection: &Connection,
    _: &Sender<ApiChannel>,
) -> Result<Response<io::Cursor<Vec<u8>>>> {
    let mut content = String::new();
    req.as_reader().read_to_string(&mut content).unwrap();

    let mut workload: WorkloadDefinition = serde_json::from_str(&content)?;
    if workload.replicas.is_none() {
        workload.replicas = Some(1);
    }
    let namespace = "default";
    let name = format!(
        "/workload/{}/{}/{}",
        workload.kind, namespace, workload.name
    );

    // Check name is not used
    if let Ok(_) = RikRepository::check_duplicate_name(connection, &name) {
        warn!("Name already used");
        return Ok(tiny_http::Response::from_string("Name already used")
            .with_status_code(tiny_http::StatusCode::from(404)));
    }

    if let Ok(inserted_id) = RikRepository::insert(
        connection,
        &name,
        &serde_json::to_string(&workload).unwrap(),
    ) {
        let workload_id: OnlyId = OnlyId { id: inserted_id };
        info!("Workload {} successfully created", &workload_id.id);
        Ok(
            tiny_http::Response::from_string(serde_json::to_string(&workload_id).unwrap())
                .with_header(tiny_http::Header::from_str("Content-Type: application/json").unwrap())
                .with_status_code(tiny_http::StatusCode::from(200)),
        )
    } else {
        error!("Cannot create workload");
        Ok(tiny_http::Response::from_string("Cannot create workload")
            .with_status_code(tiny_http::StatusCode::from(500)))
    }
}

pub fn delete(
    req: &mut tiny_http::Request,
    _: &route_recognizer::Params,
    connection: &Connection,
    internal_sender: &Sender<ApiChannel>,
) -> Result<Response<io::Cursor<Vec<u8>>>> {
    let mut content = String::new();
    req.as_reader().read_to_string(&mut content).unwrap();
    let OnlyId { id: delete_id } = serde_json::from_str(&content)?;

    if let Ok(workload) = RikRepository::find_one(connection, &delete_id, "/workload") {
        let definition: WorkloadDefinition = serde_json::from_value(workload.value).unwrap();
        internal_sender
            .send(ApiChannel {
                action: CRUD::Delete,
                workload_id: Some(delete_id),
                workload_definition: Some(definition),
                instance_id: None,
            })
            .unwrap();
        RikRepository::delete(connection, &workload.id).unwrap();

        info!("Delete workload");
        Ok(tiny_http::Response::from_string("").with_status_code(tiny_http::StatusCode::from(204)))
    } else {
        error!("Workload id {} not found", delete_id);
        Ok(
            tiny_http::Response::from_string(format!("Workload id {} not found", delete_id))
                .with_status_code(tiny_http::StatusCode::from(404)),
        )
    }
}
