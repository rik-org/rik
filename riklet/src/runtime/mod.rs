pub mod network;

pub mod function_runtime;
pub mod pod_runtime;

use self::{
    function_runtime::FunctionRuntimeManager, network::NetworkError, pod_runtime::PodRuntimeManager,
};
use crate::{cli::config::Configuration, structs::WorkloadDefinition};
use async_trait::async_trait;
use firepilot::{builder::BuilderError, machine::FirepilotError};
use proto::worker::InstanceScheduling;
use std::fmt::Debug;
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Runtime error: {0}")]
    Error(String),

    #[error("Network error: {0}")]
    NetworkError(NetworkError),

    #[error("Fetching error: {0}")]
    FetchingError(curl::Error),

    #[error("IO error: {0}")]
    IoError(std::io::Error),

    #[error("Parsing error: {0}")]
    ParsingError(serde_json::Error),

    #[error("OCI error: {0}")]
    OciError(oci::Error),

    #[error("CRI error: {0}")]
    CriError(cri::Error),

    #[error("Firecracker error: {0:?}")]
    FirecrackerError(FirepilotError),

    #[error("Could not configure properly: {0:?}")]
    FirepilotConfiguration(BuilderError),

    #[error("Runtime expected to be running: {0}")]
    NotRunning(String),
}

type Result<T> = std::result::Result<T, RuntimeError>;

#[async_trait]
pub trait Runtime: Send + Sync {
    async fn up(&mut self) -> Result<()>;
    async fn down(&mut self) -> Result<()>;
}

#[async_trait]
pub trait RuntimeManager: Send + Sync {
    fn create_runtime(
        &self,
        workload: InstanceScheduling,
        config: Configuration,
    ) -> Result<Box<dyn Runtime>>;

    /// Generate a new runtime and run it
    async fn run_instance(
        &self,
        workload: &InstanceScheduling,
        config: Configuration,
    ) -> Result<Box<dyn Runtime>> {
        let mut runtime = self.create_runtime(workload.clone(), config.clone())?;
        runtime.up().await?;

        Ok(runtime)
    }
}

enum WorkloadKind {
    Function,
    Pod,
}

impl From<String> for WorkloadKind {
    fn from(kind: String) -> Self {
        match kind.as_str() {
            "Function" => WorkloadKind::Function,
            "Pod" => WorkloadKind::Pod,
            _ => panic!("Unknown workload kind"),
        }
    }
}

pub struct RuntimeConfigurator {}
pub type DynamicRuntimeManager<'a> = &'a dyn RuntimeManager;
impl RuntimeConfigurator {
    pub fn create(workload_definition: &WorkloadDefinition) -> DynamicRuntimeManager {
        match workload_definition.kind.clone().into() {
            WorkloadKind::Function => &FunctionRuntimeManager {},
            WorkloadKind::Pod => &PodRuntimeManager {},
        }
    }
}
