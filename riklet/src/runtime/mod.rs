pub mod function_runtime;
pub mod pod_runtime;

use self::{function_runtime::FunctionRuntimeManager, pod_runtime::PodRuntimeManager};
use crate::{cli::config::Configuration, structs::WorkloadDefinition};
use async_trait::async_trait;
use proto::worker::InstanceScheduling;
use shared::utils::ip_allocator::IpAllocator;
use std::net::Ipv4Addr;

#[derive(Debug, Clone)]
pub struct NetworkDefinition {
    pub mask_long: String,
    pub firecracker_ip: Ipv4Addr,
    pub tap_ip: Ipv4Addr,
}

pub trait Network {
    fn init(&self) -> NetworkDefinition;
}

#[async_trait]
pub trait Runtime {
    async fn run(&mut self, network_definition: &NetworkDefinition);
}

pub trait RuntimeManager {
    fn create_network(
        &self,
        workload: InstanceScheduling,
        ip_allocator: IpAllocator,
    ) -> Box<dyn Network>;
    fn create_runtime(
        &self,
        workload: InstanceScheduling,
        config: Configuration,
    ) -> Box<dyn Runtime>;

    fn create(
        &self,
        workload: &InstanceScheduling,
        ip_allocator: IpAllocator,
        config: Configuration,
    ) {
        let network = self.create_network(workload.clone(), ip_allocator.clone());
        let mut runtime = self.create_runtime(workload.clone(), config.clone());

        let network_definition = network.init();
        runtime.run(&network_definition);
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
