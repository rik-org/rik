use crate::cli::config::{Configuration, ConfigurationError};
use crate::emitters::metrics_emitter::MetricsEmitter;
use crate::runtime::network::{GlobalRuntimeNetwork, NetworkError, RuntimeNetwork};
use crate::runtime::{DynamicRuntimeManager, Runtime, RuntimeConfigurator, RuntimeError};
use crate::structs::WorkloadDefinition;
use crate::traits::EventEmitter;
use crate::utils::banner;
use definition::InstanceStatus;
use proto::common::WorkerRegistration;
use proto::worker::worker_client::WorkerClient;
use proto::worker::InstanceScheduling;
use proto::{WorkerStatus, WorkloadAction};
use std::collections::HashMap;

use thiserror::Error;
use tonic::{transport::Channel, Request, Streaming};
use tracing::{debug, error, event, info, Level};

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

    #[error("Runtime error: {0}")]
    RuntimeManagerError(RuntimeError),

    #[error("Network error: {0}")]
    NetworkError(NetworkError),

    #[error("Invalid input given: {0}")]
    InvalidInput(String),
}
type Result<T> = std::result::Result<T, RikletError>;

pub struct Riklet {
    config: Configuration,
    hostname: String,
    client: WorkerClient<Channel>,
    stream: Streaming<InstanceScheduling>,
    // Can be pod or function runtimes
    // The key is the instance id
    runtimes: HashMap<String, Box<dyn Runtime>>,
}

impl Riklet {
    async fn handle_workload(&mut self, workload: &InstanceScheduling) -> Result<()> {
        info!(
            "Instance scheduling received for instance: {}",
            &workload.instance_id
        );
        let workload_definition: WorkloadDefinition =
            serde_json::from_str(workload.definition.as_str())
                .map_err(RikletError::WorkloadParseError)?;

        let dynamic_runtime_manager: DynamicRuntimeManager =
            RuntimeConfigurator::create(&workload_definition);

        match &workload.action.into() {
            WorkloadAction::CREATE => {
                self.create_workload(workload, dynamic_runtime_manager)
                    .await?
            }
            WorkloadAction::DELETE => self.delete_workload(workload).await?,
        };

        Ok(())
    }

    async fn create_workload(
        &mut self,
        workload: &InstanceScheduling,
        dynamic_runtime_manager: DynamicRuntimeManager<'_>,
    ) -> Result<()> {
        let instance_id: &String = &workload.instance_id;
        self.send_status(InstanceStatus::Creating, instance_id)
            .await?;

        match dynamic_runtime_manager
            .run_instance(workload, self.config.clone())
            .await
        {
            Err(e) => {
                self.send_status(InstanceStatus::Failed, instance_id)
                    .await
                    .unwrap_or_else(|e| {
                        error!("Error while sending status: {}", e);
                    });
                return Err(RikletError::RuntimeManagerError(e));
            }
            Ok(runtime) => {
                self.runtimes.insert(instance_id.clone(), runtime);

                self.send_status(InstanceStatus::Running, instance_id)
                    .await?;
            }
        }
        Ok(())
    }

    /// Deletes an instance and its runtime
    ///
    /// Expected lifecycle is:
    /// Receive delete request -> Send destroying status
    /// -> Destroy instance & Unregister runtime -> Send terminated status
    #[tracing::instrument(skip_all, fields(instance_id = %workload.instance_id))]
    async fn delete_workload(&mut self, workload: &InstanceScheduling) -> Result<()> {
        debug!("Delete workload");
        let instance_id: &String = &workload.instance_id;

        let instance = self
            .runtimes
            .get_mut(instance_id)
            .ok_or_else(|| RikletError::InvalidInput(instance_id.clone()))?;

        instance
            .down()
            .await
            .map_err(RikletError::RuntimeManagerError)?;

        self.send_status(InstanceStatus::Terminated, instance_id)
            .await?;

        self.runtimes.remove(instance_id);
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(instance_id = %instance_id, status = %status))]
    async fn send_status(&self, status: InstanceStatus, instance_id: &str) -> Result<()> {
        info!("Update instance status");

        let status = WorkerStatus::new(self.hostname.clone(), instance_id.to_string(), status);

        MetricsEmitter::emit_event(self.client.clone(), vec![status.0])
            .await
            .unwrap_or_else(|err| event!(Level::ERROR, "Error while sending status : {:?}", err));
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        self.start_metrics_updater();
        info!("Riklet is running");

        while let Some(workload) = self
            .stream
            .message()
            .await
            .map_err(RikletError::MessageStatusError)?
        {
            self.handle_workload(&workload).await.unwrap_or_else(|e| {
                error!("Error while handling workload: {}", e);
            })
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

        let mut client = WorkerClient::connect(config.master_ip.clone())
            .await
            .map_err(RikletError::ConnectionError)?;
        event!(Level::DEBUG, "gRPC WorkerClient connected.");

        event!(Level::DEBUG, "Node's registration to the master");
        let request = Request::new(WorkerRegistration {
            hostname: hostname.clone(),
        });
        let stream = client.register(request).await.unwrap().into_inner();

        let mut global_runtime_network = GlobalRuntimeNetwork::new();
        global_runtime_network
            .init()
            .await
            .map_err(RikletError::NetworkError)?;

        Ok(Self {
            hostname,
            client,
            stream,
            runtimes: HashMap::<String, Box<dyn Runtime>>::new(),
            config,
        })
    }
}
