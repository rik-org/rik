pub mod external;
pub mod types;

use definition::workload::WorkloadDefinition;
use std::fmt::{Debug, Display, Formatter, Result};

#[derive(Debug)]
pub enum Crud {
    Create = 0,
    Delete = 1,
}

impl From<i32> for Crud {
    fn from(value: i32) -> Self {
        match value {
            0 => Crud::Create,
            1 => Crud::Delete,
            _ => panic!("Invalid CRUD value"),
        }
    }
}

#[derive(Debug)]
pub enum RikError {
    IoError(std::io::Error),
    HttpRequestError(serde_json::Error),
    InternalCommunicationError(String),
    InvalidName(String),
}
impl Display for RikError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            RikError::IoError(ref e) => write!(f, "{}", e),
            RikError::HttpRequestError(ref e) => write!(f, "{}", e),
            RikError::InternalCommunicationError(ref e) => write!(f, "{}", e),
            RikError::InvalidName(ref e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for RikError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match *self {
            RikError::IoError(ref e) => Some(e),
            RikError::HttpRequestError(ref e) => Some(e),
            // TODO: Implement other errors
            _ => None,
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
    pub action: Crud,
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
