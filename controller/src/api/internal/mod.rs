use crate::api::types::instance::InstanceStatus;
use crate::api::{ApiChannel, CRUD};
use crate::database::RikDataBase;
use crate::database::RikRepository;
use crate::instance::Instance;
use backoff::ExponentialBackoff;
use definition::workload::WorkloadDefinition;
use dotenv::dotenv;
use proto::common::worker_status::Status;
use proto::controller::controller_client::ControllerClient;
use proto::controller::WorkloadScheduling;
use rusqlite::Connection;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use tracing::{event, Level};
use uuid::Uuid;

#[derive(Clone)]
struct RikControllerClient {
    client: ControllerClient<tonic::transport::Channel>,
}

#[allow(dead_code)]
impl RikControllerClient {
    pub async fn connect() -> Result<RikControllerClient, tonic::transport::Error> {
        dotenv().ok();
        let scheduler_url = match std::env::var("SCHEDULER_URL") {
            Ok(val) => val,
            Err(_e) => "http://127.0.0.1:4996".to_string(),
        };
        let client = ControllerClient::connect(scheduler_url).await?;
        Ok(RikControllerClient { client })
    }

    pub async fn schedule_instance(
        &mut self,
        instance: WorkloadScheduling,
    ) -> Result<(), tonic::Status> {
        let request = tonic::Request::new(instance);
        self.client.schedule_instance(request).await?;
        Ok(())
    }

    pub async fn get_status_updates(
        &mut self,
        database: Arc<RikDataBase>,
    ) -> Result<(), tonic::Status> {
        let connection: Connection = database.open().unwrap();
        let request = tonic::Request::new(());
        let mut stream = self.client.get_status_updates(request).await?.into_inner();
        while let Some(worker_status) = stream.message().await? {
            let status = match worker_status.status {
                Some(status) => status,
                None => {
                    event!(
                        Level::ERROR,
                        "Received status update request without status from {}",
                        worker_status.identifier
                    );
                    continue;
                }
            };
            let instance_metric = match status.clone() {
                Status::Instance(instance_metric) => instance_metric,
                Status::Worker(_worker_metric) => continue,
            };
            let new_instance_status = InstanceStatus::new(instance_metric.status as usize);
            let instance_id = instance_metric.instance_id;

            let mut instance_state: Instance = match RikRepository::check_duplicate_name(
                &connection,
                &format!("/instance/%/default/{}", instance_id),
            ) {
                Ok(previous_instance) => {
                    let instance: Instance =
                        serde_json::from_value(previous_instance.value).unwrap();
                    event!(
                        Level::INFO,
                        "Instance {}, status update, {} -> {}",
                        instance_id,
                        instance.status,
                        new_instance_status
                    );
                    instance
                }
                Err(_e) => {
                    event!(
                        Level::ERROR,
                        "Instance {} could not be found from {}",
                        instance_id,
                        worker_status.identifier
                    );
                    continue;
                }
            };

            instance_state.status = instance_metric.status.into();
            let value = serde_json::to_string(&instance_state).unwrap();
            match RikRepository::upsert(
                &connection,
                &instance_id,
                &instance_state.get_full_name(),
                &value,
                "/instance",
            ) {
                Ok(value) => value,
                Err(e) => panic!("{:?}", e),
            };
        }
        Ok(())
    }
}

#[allow(dead_code)]
pub struct Server {
    external_sender: Sender<ApiChannel>,
    internal_receiver: Receiver<ApiChannel>,
}

impl Server {
    pub fn new(
        external_sender: Sender<ApiChannel>,
        internal_receiver: Receiver<ApiChannel>,
    ) -> Server {
        Server {
            external_sender,
            internal_receiver,
        }
    }

    pub async fn run(&self, database: Arc<RikDataBase>) {
        let client: RikControllerClient = RikControllerClient::connect().await.unwrap();

        let mut client_clone = client.clone();
        let database = database.clone();

        tokio::spawn(async move {
            if let Err(e) = client_clone.get_status_updates(database).await {
                event!(
                    Level::ERROR,
                    "Internal communication with scheduler failed: {:?}",
                    e
                );
            }
        });

        self.listen_notification(client).await;
    }

    async fn handle_create(
        &self,
        instance: Instance,
        workload_def: WorkloadDefinition,
        client: &mut RikControllerClient,
    ) {
        event!(Level::INFO, "Schedule instance {}", instance.id);
        client
            .schedule_instance(WorkloadScheduling {
                workload_id: instance.workload_id,
                definition: serde_json::to_string(&workload_def).unwrap(),
                action: CRUD::Create as i32,
                instance_id: instance.id,
            })
            .await
            .unwrap();
    }

    async fn handle_delete(
        &self,
        instance: Instance,
        workload_def: WorkloadDefinition,
        client: &mut RikControllerClient,
    ) {
        event!(Level::INFO, "Unschedule instance {}", instance.id);
        client
            .schedule_instance(WorkloadScheduling {
                workload_id: instance.workload_id,
                definition: serde_json::to_string(&workload_def).unwrap(),
                action: CRUD::Delete as i32,
                instance_id: instance.id,
            })
            .await
            .unwrap();
    }

    async fn listen_notification(&self, mut client: RikControllerClient) {
        for notification in &self.internal_receiver {
            match notification.action {
                CRUD::Create => {
                    let definition = notification.workload_definition.as_ref().unwrap().clone();
                    let instance: Instance = notification.into();
                    self.handle_create(instance, definition, &mut client).await;
                }
                CRUD::Delete => {
                    let definition = notification.workload_definition.as_ref().unwrap().clone();
                    let instance: Instance = notification.into();
                    self.handle_delete(instance, definition, &mut client).await;
                }
            }
        }
    }
}
