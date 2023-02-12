pub mod external;
pub mod internal;
pub mod types;

use definition::workload::WorkloadDefinition;
use std::fmt::{Display, Formatter, Result};
#[allow(dead_code)]
#[derive(Debug)]
pub enum CRUD {
    Create = 0,
    Delete = 1,
}

#[derive(Debug)]
pub enum RikError {
    IoError(std::io::Error),
    HttpRequestError(serde_json::Error),
}
impl Display for RikError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            RikError::IoError(ref e) => e.fmt(f),
            RikError::HttpRequestError(ref e) => e.fmt(f),
        }
    }
}

impl std::error::Error for RikError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match *self {
            RikError::IoError(ref e) => Some(e),
            RikError::HttpRequestError(ref e) => Some(e),
        }
    }
}

impl From<std::io::Error> for RikError {
    fn from(e: std::io::Error) -> RikError {
        RikError::IoError(e)
    }
}

impl From<serde_json::Error> for RikError {
    fn from(e: serde_json::Error) -> RikError {
        RikError::HttpRequestError(e)
    }
}

pub struct ApiChannel {
    pub action: CRUD,
    pub workload_id: Option<String>,
    pub instance_id: Option<String>,
    pub workload_definition: Option<WorkloadDefinition>,
}
impl Display for ApiChannel {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "Action: {:?}, Workload id: {:?}, Instance id: {:?}",
            self.action, self.workload_id, self.instance_id
        )
    }
}
