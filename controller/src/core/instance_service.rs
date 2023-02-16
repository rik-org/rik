use crate::api::{RikError, CRUD};
use crate::core::core::CoreInternalEvent;
use crate::core::instance::{Instance, InstanceStatus};
use crate::core::instance_repository::InstanceRepositoryImpl;
use crate::core::{with_backoff, InstanceRepository, InstanceService, Listener};
use async_trait::async_trait;
use definition::workload::WorkloadDefinition;
use dotenv::dotenv;
use proto::common::InstanceMetric;
use proto::controller::controller_client::ControllerClient;
use proto::controller::WorkloadScheduling;
use std::sync::mpsc::Sender;
use tracing::{event, Level};

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
            while let Some(status) = stream.message().await.unwrap() {
                sender
                    .send(CoreInternalEvent::SchedulerNotification(status))
                    .unwrap();
            }
            ()
        });
    }
}

impl InstanceServiceImpl {
    pub(crate) async fn new(
        service: InstanceRepositoryImpl,
        sender: Sender<CoreInternalEvent>,
    ) -> Result<InstanceServiceImpl, RikError> {
        dotenv().ok();
        let scheduler_url = match std::env::var("SCHEDULER_URL") {
            Ok(val) => val,
            Err(_e) => "http://127.0.0.1:4996".to_string(),
        };

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
        action: CRUD,
    ) -> Result<(), tonic::Status> {
        let scheduling = WorkloadScheduling {
            workload_id: instance.workload_id,
            definition: serde_json::to_string(&workload_def).unwrap(),
            action: action as i32,
            instance_id: instance.id,
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
        instance: Instance,
        workload_def: WorkloadDefinition,
    ) -> Result<(), RikError> {
        event!(Level::INFO, "Schedule instance {}", instance.id);
        self.schedule_instance(instance, workload_def, CRUD::Create)
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
        self.schedule_instance(instance, workload_def, CRUD::Delete)
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
