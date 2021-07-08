use crate::api;
use crate::api::external::services::element::elements_set_right_name;
use crate::api::types::element::OnlyId;
use crate::api::{ApiChannel, CRUD};
use crate::database::RikRepository;
use crate::logger::{LogType, LoggingChannel};

use definition::workload::WorkloadDefinition;
use route_recognizer;
use rusqlite::Connection;
use std::io;
use std::str::FromStr;
use std::sync::mpsc::Sender;

pub fn get(
    _: &mut tiny_http::Request,
    _: &route_recognizer::Params,
    connection: &Connection,
    _: &Sender<ApiChannel>,
    logger: &Sender<LoggingChannel>,
) -> Result<tiny_http::Response<io::Cursor<Vec<u8>>>, api::RikError> {
    if let Ok(mut workloads) = RikRepository::find_all(connection, "/workload") {
        workloads = elements_set_right_name(workloads.clone());
        let workloads_json = serde_json::to_string(&workloads).unwrap();
        logger
            .send(LoggingChannel {
                message: String::from("Workloads found"),
                log_type: LogType::Log,
            })
            .unwrap();

        Ok(tiny_http::Response::from_string(workloads_json)
            .with_header(tiny_http::Header::from_str("Content-Type: application/json").unwrap())
            .with_status_code(tiny_http::StatusCode::from(200)))
    } else {
        Ok(tiny_http::Response::from_string("Cannot find workloads")
            .with_status_code(tiny_http::StatusCode::from(500)))
    }
}

pub fn create(
    req: &mut tiny_http::Request,
    _: &route_recognizer::Params,
    connection: &Connection,
    _: &Sender<ApiChannel>,
    logger: &Sender<LoggingChannel>,
) -> Result<tiny_http::Response<io::Cursor<Vec<u8>>>, api::RikError> {
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
        logger
            .send(LoggingChannel {
                message: String::from("Name already used"),
                log_type: LogType::Warn,
            })
            .unwrap();
        return Ok(tiny_http::Response::from_string("Name already used")
            .with_status_code(tiny_http::StatusCode::from(404)));
    }

    if let Ok(inserted_id) = RikRepository::insert(
        connection,
        &name,
        &serde_json::to_string(&workload).unwrap(),
    ) {
        let workload_id: OnlyId = OnlyId { id: inserted_id };
        logger
            .send(LoggingChannel {
                message: String::from(format!("Workload {} successfully created", &workload_id.id)),
                log_type: LogType::Log,
            })
            .unwrap();
        Ok(
            tiny_http::Response::from_string(serde_json::to_string(&workload_id).unwrap())
                .with_header(tiny_http::Header::from_str("Content-Type: application/json").unwrap())
                .with_status_code(tiny_http::StatusCode::from(200)),
        )
    } else {
        logger
            .send(LoggingChannel {
                message: String::from("Cannot create workload"),
                log_type: LogType::Error,
            })
            .unwrap();
        Ok(tiny_http::Response::from_string("Cannot create workload")
            .with_status_code(tiny_http::StatusCode::from(500)))
    }
}

pub fn delete(
    req: &mut tiny_http::Request,
    _: &route_recognizer::Params,
    connection: &Connection,
    internal_sender: &Sender<ApiChannel>,
    logger: &Sender<LoggingChannel>,
) -> Result<tiny_http::Response<io::Cursor<Vec<u8>>>, api::RikError> {
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

        logger
            .send(LoggingChannel {
                message: String::from("Delete workload"),
                log_type: LogType::Log,
            })
            .unwrap();
        Ok(tiny_http::Response::from_string("").with_status_code(tiny_http::StatusCode::from(204)))
    } else {
        logger
            .send(LoggingChannel {
                message: String::from(format!("Workload id {} not found", delete_id)),
                log_type: LogType::Error,
            })
            .unwrap();
        Ok(
            tiny_http::Response::from_string(format!("Workload id {} not found", delete_id))
                .with_status_code(tiny_http::StatusCode::from(404)),
        )
    }
}
