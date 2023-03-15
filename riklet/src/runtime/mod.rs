pub mod function_runtime;
pub mod pod_runtime;

use self::{function_runtime::FunctionRuntimeManager, pod_runtime::PodRuntimeManager};
use crate::{cli::config::Configuration, iptables::IptablesError, structs::WorkloadDefinition};
use async_trait::async_trait;
use firepilot::FirecrackerError;
use proto::worker::InstanceScheduling;
use shared::utils::ip_allocator::IpAllocator;
use std::{fmt::Debug, net::Ipv4Addr};
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

#[derive(Debug, Clone)]
pub struct NetworkDefinition {
    pub mask_long: String,
    pub firecracker_ip: Ipv4Addr,
    pub tap_ip: Ipv4Addr,
}

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Network error: {0}")]
    CommonNetworkError(String),

    #[error("IO error: {0}")]
    IoError(std::io::Error),

    #[error("Iptables error: {0}")]
    Iptables(IptablesError),
}
type NetworkResult<T> = std::result::Result<T, NetworkError>;

#[async_trait]
pub trait Network: Send + Sync {
    fn init(&self) -> NetworkResult<NetworkDefinition>;
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Firecracker error: {0}")]
    Firecracker(FirecrackerError),

    #[error("OCI error: {0}")]
    OCI(oci::Error),

    #[error("CRI error: {0}")]
    CRI(cri::Error),
}
type RuntimeResult<T> = std::result::Result<T, RuntimeError>;
#[async_trait]
pub trait Runtime: Send + Sync + Debug {
    async fn run(&mut self, network_definition: &NetworkDefinition) -> RuntimeResult<()>;
}

#[async_trait]
pub trait RuntimeManager: Send + Sync {
    fn create_network(
        &self,
        workload: InstanceScheduling,
        ip_allocator: IpAllocator,
    ) -> Result<Box<dyn Network>>;
    fn create_runtime(
        &self,
        workload: InstanceScheduling,
        config: Configuration,
    ) -> Result<Box<dyn Runtime>>;

    async fn create(
        &self,
        workload: &InstanceScheduling,
        ip_allocator: IpAllocator,
        config: Configuration,
    ) -> Result<Box<dyn Runtime>> {
        let network = self.create_network(workload.clone(), ip_allocator.clone())?;
        let mut runtime = self.create_runtime(workload.clone(), config.clone())?;

        let network_definition = network.init().map_err(RuntimeManagerError::Network)?;
        runtime
            .run(&network_definition)
            .await
            .map_err(RuntimeManagerError::Runtime)?;

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
            "FUNCTION" => WorkloadKind::FUNCTION,
            "POD" => WorkloadKind::POD,
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
