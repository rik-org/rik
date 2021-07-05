use httpclient::api::ApiRequest;
use httpclient::ApiError;
use prettytable::{format, Cell, Row, Table};

use serde_json::Value;
use std::fs;

#[derive(Debug)]
pub struct WorkloadService {}

const ENDPOINT: &'static str = "api/v0/workloads.";

impl WorkloadService {
    pub fn list() -> Result<(), ApiError> {
        let api_request: ApiRequest = ApiRequest::new(format!("{}{}", ENDPOINT, "list"), None)?;
        let workloads = api_request.get()?;
        let mut table = Table::new();
        table.set_titles(row!["id", "name", "kind"]);
        table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
        for workload in workloads.iter() {
            let id = &workload["id"];
            let values = workload.get("value").unwrap();
            table.add_row(Row::new(vec![
                Cell::new(id.as_str().unwrap()),
                Cell::new(values.get("name").unwrap().as_str().unwrap()),
                Cell::new(values.get("kind").unwrap().as_str().unwrap()),
            ]));
        }
        table.printstd();
        Ok(())
    }

    pub fn create(pathfile: &str) -> Result<Value, ApiError> {
        match fs::read_to_string(pathfile) {
            Ok(body) => {
                let api_request: ApiRequest =
                    ApiRequest::new(format!("{}{}", ENDPOINT, "create"), Some(body))?;
                api_request.post()
            }
            Err(_) => Err(ApiError::BadConfigFile),
        }
    }

    pub fn delete(id: String) -> Result<Value, ApiError> {
        let body = format!(r#"{{"id": "{}"}}"#, id);
        let api_request: ApiRequest =
            ApiRequest::new(format!("{}{}", ENDPOINT, "delete"), Some(body))?;
        api_request.post()
    }
}

#[cfg(test)]
mod tests {
    use crate::services::workload_service::WorkloadService;
    use crate::ApiError;

    #[test]
    fn create_with_invalid_filepath_return_error() -> Result<(), ApiError> {
        assert!(WorkloadService::create("nimportequoi.xhtml").is_err());
        Ok(())
    }
}
