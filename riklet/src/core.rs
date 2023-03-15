use crate::cli::config::{Configuration, ConfigurationError};
use crate::emitters::metrics_emitter::MetricsEmitter;
use crate::iptables::rule::Rule;
use crate::iptables::{Chain, Iptables, MutateIptables, Table};
use crate::network::net::{Net, NetworkInterfaceConfig};
use crate::structs::{Container, WorkloadDefinition};
use crate::runtime::{DynamicRuntimeManager, Runtime, RuntimeConfigurator, RuntimeManagerError};
use crate::traits::EventEmitter;
use crate::utils::banner;
use ipnetwork::Ipv4Network;
use proto::common::{InstanceMetric, WorkerRegistration, WorkerStatus};
use proto::worker::worker_client::WorkerClient;
use proto::worker::InstanceScheduling;
use proto::InstanceStatus;
use shared::utils::ip_allocator::IpAllocator;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};

use std::time::Duration;
use std::{fs, io, thread};
use std::ops::Deref;
use thiserror::Error;
use tonic::{transport::Channel, Request, Streaming};
use tracing::{debug, event, Level};

const METRICS_UPDATER_INTERVAL: u64 = 15 * 1000;

#[derive(Error, Debug)]
pub enum RikletError {
    #[error("Failed to parse workload definition: {0}")]
    WorkloadParseError(serde_json::Error),

    #[error("Message status error: {0}")]
    MessageStatusError(tonic::Status),

    #[error("Configuration error: {0}")]
    ConfigurationError(ConfigurationError),

    #[error("Failed to connect client: {0}")]
    ConnectionError(tonic::transport::Error),

    #[error("Network error: {0}")]
    NetworkError(ipnetwork::IpNetworkError),

    #[error("Runtime error: {0}")]
    RuntimeManagerError(RuntimeManagerError),
}
type Result<T> = std::result::Result<T, RikletError>;

enum WorkloadAction {
    CREATE,
    DELETE,
}

struct RikletWorkerStatus(WorkerStatus);
impl RikletWorkerStatus {
    fn new(identifier: String, instance_id: String, status: InstanceStatus) -> Self {
        Self(WorkerStatus {
            identifier,
            host_address: None,
            status: Some(proto::common::worker_status::Status::Instance(
                InstanceMetric {
                    instance_id,
                    status: status.into(),
                    metrics: "".to_string(),
                },
            )),
        })
    }
}

impl Deref for RikletWorkerStatus {
    type Target = WorkerStatus;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Into<WorkloadAction> for i32 {
    fn into(self) -> WorkloadAction {
        match self {
            1 => WorkloadAction::CREATE,
            2 => WorkloadAction::DELETE,
            _ => panic!("Unknown workload action"),
        }
    }
}

#[derive(Debug)]
pub struct Riklet {
    config: Configuration,
    hostname: String,
    client: WorkerClient<Channel>,
    stream: Streaming<InstanceScheduling>,
    runtimes: HashMap<String, Box<dyn Runtime>>,
    ip_allocator: IpAllocator,
    // function_config: FnConfiguration,
}

impl Riklet {
    async fn handle_workload(&mut self, workload: &InstanceScheduling) -> Result<()> {
        event!(Level::DEBUG, "Handling workload");
        let workload_definition: WorkloadDefinition =
            serde_json::from_str(workload.definition.as_str())
                .map_err(RikletError::WorkloadParseError)?;

        let dynamic_runtime_manager: DynamicRuntimeManager =
            RuntimeConfigurator::create(&workload_definition);

        Ok(match &workload.action.into() {
            WorkloadAction::CREATE => {
                self.create_workload(workload, dynamic_runtime_manager)
                    .await?
            }
            WorkloadAction::DELETE => {
                self.delete_workload(workload, dynamic_runtime_manager)
                    .await?
            }
        })
    }

    async fn create_workload(
        &mut self,
        workload: &InstanceScheduling,
        dynamic_runtime_manager: DynamicRuntimeManager<'_>,
    ) -> Result<()> {
        event!(Level::DEBUG, "Creating workload");
        let instance_id: &String = &workload.instance_id;
        let runtime = dynamic_runtime_manager
            .create(workload, self.ip_allocator.clone(), self.config.clone())
            .await
            .map_err(RikletError::RuntimeManagerError)?;

        self.runtimes.insert(instance_id.clone(), runtime);

        self.send_status(InstanceStatus::Running, instance_id)
            .await?;
        Ok(())
    }

    async fn delete_workload(
        &mut self,
        workload: &InstanceScheduling,
        runtime: DynamicRuntimeManager<'_>,
    ) -> Result<()> {
        event!(Level::DEBUG, "Destroying workload");
        let instance_id: &String = &workload.instance_id;
        // Destroy the runtime
        // TODO

        self.runtimes.remove(instance_id);

        self.send_status(InstanceStatus::Terminated, instance_id)
            .await?;

        runtime.destroy();
        Ok(())
    }

    async fn send_status(&self, status: InstanceStatus, instance_id: &str) -> Result<()> {
        event!(Level::DEBUG, "Sending status : {}", status);

        let status =
            RikletWorkerStatus::new(self.hostname.clone(), instance_id.to_string(), status);

        MetricsEmitter::emit_event(self.client.clone(), vec![status.0])
            .await
            .unwrap_or_else(|err| event!(Level::ERROR, "Error while sending status : {:?}", err));
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        event!(Level::INFO, "Riklet is running.");
        self.start_metrics_updater();

        while let Some(workload) = self
            .stream
            .message()
            .await
            .map_err(RikletError::MessageStatusError)?
        {
            self.handle_workload(&workload).await?;
        }
        Ok(())
    }

    fn start_metrics_updater(&self) {
        event!(Level::INFO, "Starting metrics updater");
        let client = self.client.clone();
        let hostname = self.hostname.clone();

        tokio::spawn(async move {
            let mut metrics_emitter = MetricsEmitter::new(hostname.clone(), client.clone());
            metrics_emitter
                .emit_interval(METRICS_UPDATER_INTERVAL)
                .await;
        });
    }

    pub async fn new() -> Result<Self> {
        event!(Level::DEBUG, "Riklet bootstraping process started.");
        banner();
        let hostname = gethostname::gethostname().into_string().unwrap();

        let config = Configuration::load().map_err(RikletError::ConfigurationError)?;
        // let function_config = FnConfiguration::load();

        let mut client = WorkerClient::connect(config.master_ip.clone())
            .await
            .map_err(RikletError::ConnectionError)?;
        event!(Level::DEBUG, "gRPC WorkerClient connected.");

        event!(Level::DEBUG, "Node's registration to the master");
        let request = Request::new(WorkerRegistration {
            hostname: hostname.clone(),
        });
        let stream = client.register(request).await.unwrap().into_inner();

        // TODO Network
        let network = Ipv4Network::new(Ipv4Addr::new(192, 168, 1, 0), 24)
            .map_err(RikletError::NetworkError)?;
        let ip_allocator = IpAllocator::new(network);

        Ok(Self {
            hostname,
            client,
            stream,
            runtimes: HashMap::<String, Box<dyn Runtime>>::new(),
            ip_allocator,
            config,
        })
    }
