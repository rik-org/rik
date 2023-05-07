pub mod external;
pub mod types;

use definition::workload::WorkloadDefinition;
use std::fmt::{Debug, Display, Formatter, Result};
use thiserror::Error;

use crate::database::DatabaseError;

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

#[derive(Debug, Error)]
pub enum RikError {
    #[error("IO error: {0}")]
    IoError(std::io::Error),

    #[error("HTTP request error: {0}")]
    ParsingError(serde_json::Error),

    #[error("Internal communication error: {0}")]
    InternalCommunicationError(String),

    #[error("Invalid name: {0}")]
    InvalidName(String),

    #[error("Database error: {0}")]
    DataBaseError(DatabaseError),
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
