use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::PathBuf;

/// `Workload` hold all workload attributes.
#[derive(Serialize, Deserialize, Debug)]
pub struct Workload {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,
    pub name: String,
    pub spec: Spec,
}

/// `Spec` hold the workload specification.
///
/// This will be used by the system to determine the container to run, etc.
#[derive(Serialize, Deserialize, Debug)]
pub struct Spec {
    pub containers: Vec<Container>,
}

/// `Container` hold attributes for one workload container.
#[derive(Serialize, Deserialize, Debug)]
pub struct Container {
    pub name: String,
    pub image: String,
}

/// Workload related errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unable to read the workload file. Details : {0}")]
    ReadFile(std::io::Error),
    #[error("Failed to deserialize the workload. Details : {0}")]
    Deserialization(serde_json::Error),
}

/// Implementation of the `TryFrom` trait for `Workload` in order to be
/// able to load a `Workload` from a file.
impl TryFrom<PathBuf> for Workload {
    type Error = Error;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        // Try to open the workload file
        let file = File::open(value).map_err(Error::ReadFile)?;
        // Try to deserialize the workload from the file content
        serde_json::from_reader::<File, Workload>(file).map_err(Error::Deserialization)
    }
}

impl TryFrom<&str> for Workload {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        serde_json::from_str::<Workload>(value).map_err(Error::Deserialization)
    }
}
