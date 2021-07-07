use httpclient::api::ApiRequest;
use httpclient::ApiError;
use prettytable::{format, Cell, Row, Table};

use serde_json::Value;

#[derive(Debug)]
pub struct InstanceService {}

const ENDPOINT: &'static str = "api/v0/instances.";

impl InstanceService {
    pub fn list() -> Result<(), ApiError> {
        let api_request: ApiRequest =
            ApiRequest::new(format!("{}{}", ENDPOINT, "list"), None, None)?;
        let instances = api_request.get()?;
        let mut table = Table::new();
        table.set_titles(row!["id", "name", "workload_id"]);
        table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
        for instance in instances.iter() {
            let id = &instance["id"];
            let name = &instance["name"];
            let workload_id = &instance["workload_id"];
            table.add_row(Row::new(vec![
                Cell::new(id.to_string().as_str()),
                Cell::new(name.to_string().as_str()),
                Cell::new(workload_id.to_string().as_str()),
            ]));
        }
        table.printstd();
        Ok(())
    }

    pub fn create(id: String, replica: Option<String>) -> Result<Value, ApiError> {
        let nb_replicas: String;
        if replica != None {
            nb_replicas = replica.unwrap();
        } else {
            nb_replicas = "1".to_string();
        }
        let body = format!(r#"{{"workload_id": {},
        "replicas": {}}}"#, id, nb_replicas);
        let api_request: ApiRequest =
            ApiRequest::new(format!("{}{}", ENDPOINT, "create"), Some(body), None)?;
        api_request.post()
    }

    pub fn delete(id: String) -> Result<Value, ApiError> {
        let body = format!(r#"{{"id": {}}}"#, id);
        let api_request: ApiRequest =
            ApiRequest::new(format!("{}{}", ENDPOINT, "delete"), Some(body), None)?;
        api_request.post()
    }
}

#[cfg(test)]
mod tests {
    use crate::services::instance_service::InstanceService;
    use crate::ApiError;

    #[test]
    fn delete_not_found_instance_return_error() -> Result<(), ApiError> {
        assert!(InstanceService::delete(String::from("0")).is_err());
        Ok(())
    }
}
