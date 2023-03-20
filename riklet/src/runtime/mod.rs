pub mod network;

pub mod function_runtime;
pub mod pod_runtime;

use self::{
    function_runtime::FunctionRuntimeManager, network::NetworkError, pod_runtime::PodRuntimeManager,
};
use crate::{cli::config::Configuration, structs::WorkloadDefinition};
use async_trait::async_trait;
use firepilot::FirecrackerError;
use proto::worker::InstanceScheduling;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeManagerError {
    #[error("Runtime error: {0}")]
    Runtime(RuntimeError),

    #[error("Network error: {0}")]
    Network(NetworkError),

    #[error("Curl error: {0}")]
    CurlError(curl::Error),

    #[error("IO error: {0}")]
    IoError(std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(serde_json::Error),

    #[error("OCI error: {0}")]
    OCI(oci::Error),

    #[error("CRI error: {0}")]
    CRI(cri::Error),
}

type Result<T> = std::result::Result<T, RuntimeManagerError>;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Firecracker error: {0}")]
    Firecracker(FirecrackerError),

    #[error("OCI error: {0}")]
    OCI(oci::Error),

    #[error("CRI error: {0}")]
    CRI(cri::Error),

    #[error("Network error: {0}")]
    Network(NetworkError),
}
type RuntimeResult<T> = std::result::Result<T, RuntimeError>;
#[async_trait]
pub trait Runtime: Send + Sync + Debug {
    async fn run(&mut self) -> RuntimeResult<()>;
}

#[async_trait]
pub trait RuntimeManager: Send + Sync {
    // fn create_network(&self, workload: InstanceScheduling) -> Result<Box<dyn Network>>;
    fn create_runtime(
        &self,
        workload: InstanceScheduling,
        config: Configuration,
    ) -> Result<Box<dyn Runtime>>;

    async fn run(
        &self,
        workload: &InstanceScheduling,
        config: Configuration,
    ) -> Result<Box<dyn Runtime>> {
        let mut runtime = self.create_runtime(workload.clone(), config.clone())?;
        runtime.run().await.map_err(RuntimeManagerError::Runtime)?;

        Ok(runtime)
    }

    fn destroy(&self) {
        println!("Destroying runtime");
    }
}

enum WorkloadKind {
    FUNCTION,
    POD,
}

impl Into<WorkloadKind> for String {
    fn into(self) -> WorkloadKind {
        match self.as_str() {
            "Function" => WorkloadKind::FUNCTION,
            "Pod" => WorkloadKind::POD,
            _ => panic!("Unknown workload kind"),
        }
    }
}

pub struct RuntimeConfigurator {}
pub type DynamicRuntimeManager<'a> = &'a dyn RuntimeManager;
impl RuntimeConfigurator {
    pub fn create(workload_definition: &WorkloadDefinition) -> DynamicRuntimeManager {
        match workload_definition.kind.clone().into() {
            WorkloadKind::FUNCTION => &FunctionRuntimeManager {},
            WorkloadKind::POD => &PodRuntimeManager {},
        }
    }
}
