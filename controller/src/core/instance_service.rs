use crate::api::{Crud, RikError};
use crate::core::core::CoreInternalEvent;
use crate::core::instance::Instance;
use crate::core::instance_repository::InstanceRepositoryImpl;
use crate::core::{with_backoff, InstanceRepository, InstanceService, Listener};
use async_trait::async_trait;
use definition::workload::{WorkloadDefinition, WorkloadKind};
use dotenv::dotenv;
use proto::common::worker_status::Status;
use proto::common::InstanceMetric;
use proto::controller::controller_client::ControllerClient;
use proto::controller::WorkloadScheduling;
use proto::InstanceStatus;
use rand::Rng;
use std::net::SocketAddr;
use std::ops::Range;
use std::str::FromStr;
use std::sync::mpsc::Sender;
use tracing::{event, Level};

const WORKLOAD_PORTS: Range<u16> = 45000..50000;
const DEFAULT_SCHEDULER_URL: &str = "http://localhost:4996";

pub fn mutate_function_port(mut workload: WorkloadDefinition) -> WorkloadDefinition {
    let random_port = rand::thread_rng().gen_range(WORKLOAD_PORTS);
    workload.set_function_port(random_port);
    workload
}

pub struct InstanceServiceImpl {
    client: ControllerClient<tonic::transport::Channel>,
    sender: Sender<CoreInternalEvent>,
    service: InstanceRepositoryImpl,
}

impl Listener for InstanceServiceImpl {
    fn run_listen_thread(&mut self) {
        let mut client = self.client.clone();
        let sender = self.sender.clone();
        tokio::spawn(async move {
            let mut stream = client.get_status_updates(()).await.unwrap().into_inner();
            while let Some(notification) = stream.message().await.unwrap() {
                let status = notification.status.unwrap();
                match status {
                    Status::Instance(metric) => {
                        event!(
                            Level::INFO,
                            "Instance status update: {}",
                            &notification.identifier
                        );
                        sender
                            .send(CoreInternalEvent::InstanceStatusUpdate(metric))
                            .unwrap();
                    }
                    Status::Worker(metric) => {
                        sender
                            .send(CoreInternalEvent::WorkerStatusUpdate {
                                identifier: notification.identifier,
                                address: SocketAddr::from_str(
                                    notification.host_address.unwrap().as_str(),
                                )
                                .unwrap(),
                                metric,
                            })
                            .unwrap();
                    }
                }
            }
        });
    }
}

impl InstanceServiceImpl {
    pub(crate) async fn new(
        service: InstanceRepositoryImpl,
        sender: Sender<CoreInternalEvent>,
    ) -> Result<InstanceServiceImpl, RikError> {
        dotenv().ok();
        let scheduler_url =
            std::env::var("SCHEDULER_URL").unwrap_or_else(|_| DEFAULT_SCHEDULER_URL.to_string());

        let controller_client =
            with_backoff(|| async { Ok(ControllerClient::connect(scheduler_url.clone()).await?) })
                .await?;
        let client = InstanceServiceImpl {
            client: controller_client,
            sender,
            service,
        };

        Ok(client)
    }

    async fn schedule_instance(
        &mut self,
        instance: Instance,
        workload_def: WorkloadDefinition,
        action: Crud,
    ) -> Result<(), tonic::Status> {
        let scheduling = WorkloadScheduling {
            workload_id: instance.workload_id.clone(),
            definition: serde_json::to_string(&workload_def).unwrap(),
            action: action as i32,
            instance_id: instance.id.clone(),
        };
        let request = tonic::Request::new(scheduling);
        self.client.schedule_instance(request).await?;
        Ok(())
    }
}

#[async_trait]
impl InstanceService for InstanceServiceImpl {
    async fn create_instance(
        &mut self,
        mut instance: Instance,
        mut workload_def: WorkloadDefinition,
    ) -> Result<(), RikError> {
        event!(Level::INFO, "Schedule instance {}", instance.id);

        if instance.kind == WorkloadKind::Function {
            workload_def = mutate_function_port(workload_def);
        }

        instance.spec = workload_def.spec.clone();
        self.service.register_instance(instance.clone())?;
        self.schedule_instance(instance, workload_def, Crud::Create)
            .await
            .map_err(|e| {
                RikError::InternalCommunicationError(format!("Could not schedule instance: {}", e))
            })
    }
    async fn delete_instance(
        &mut self,
        instance: Instance,
        workload_def: WorkloadDefinition,
    ) -> Result<(), RikError> {
        event!(Level::INFO, "Unschedule instance {}", instance.id);
        self.schedule_instance(instance, workload_def, Crud::Delete)
            .await
            .map_err(|e| {
                RikError::InternalCommunicationError(format!("Could not schedule instance: {}", e))
            })
    }

    fn handle_instance_status_update(&mut self, instance_metric: InstanceMetric) {
        let new_status = InstanceStatus::from(instance_metric.status);
        let mut instance = self
            .service
            .fetch_instance(instance_metric.instance_id)
            .unwrap();
        event!(
            Level::INFO,
            "Instance {}, status update, {} -> {}",
            instance.id,
            instance.status,
            &new_status
        );

        instance.status = new_status;
        self.service.register_instance(instance).unwrap();
    }
}
