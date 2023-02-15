use crate::api::{ApiChannel, CRUD};
use crate::database::RikDataBase;
use crate::database::RikRepository;
use crate::instance::Instance;
use anyhow::{Context, Result};
use colored::Colorize;
use definition::workload::WorkloadDefinition;
use dotenv::dotenv;
use log::info;
use proto::common::worker_status::Status;
use proto::controller::controller_client::ControllerClient;
use proto::controller::WorkloadScheduling;
use rusqlite::Connection;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;

#[derive(Clone)]
struct RikControllerClient {
    client: ControllerClient<tonic::transport::Channel>,
}

#[allow(dead_code)]
impl RikControllerClient {
    pub async fn connect() -> Result<RikControllerClient> {
        dotenv().ok();
        let scheduler_url = match std::env::var("SCHEDULER_URL") {
            Ok(val) => val,
            Err(_e) => "http://127.0.0.1:4996".to_string(),
        };
        let client = ControllerClient::connect(scheduler_url)
            .await
            .context("Fail to connect to scheduler")?;
        info!("{}", "Connected to scheduler".green());
        Ok(RikControllerClient { client })
    }

    pub async fn schedule_instance(&mut self, instance: WorkloadScheduling) -> Result<()> {
        let request = tonic::Request::new(instance);
        self.client.schedule_instance(request).await?;
        Ok(())
    }

    pub async fn get_status_updates(&mut self, database: Arc<RikDataBase>) -> Result<()> {
        let connection: Connection = database.open().unwrap();
        let request = tonic::Request::new(());
        let mut stream = self.client.get_status_updates(request).await?.into_inner();
        while let Some(worker_status) = stream.message().await? {
            println!("Received status update request {:?}", worker_status);
            let status = match worker_status.status {
                Some(status) => status,
                None => {
                    println!("Received status update request without status");
                    continue;
                }
            };
            let instance_metric = match status.clone() {
                Status::Instance(instance_metric) => instance_metric,
                Status::Worker(_worker_metric) => continue,
            };
            let instance_status = instance_metric.status;
            let instance_id = instance_metric.instance_id;

            let mut instance_state: Instance = match RikRepository::check_duplicate_name(
                &connection,
                &format!("/instance/%/default/{}", instance_id),
            ) {
                Ok(previous_instance) => serde_json::from_value(previous_instance.value).unwrap(),
                Err(_e) => {
                    println!("Instance {} not found", instance_id);
                    continue;
                }
            };

            instance_state.status = instance_status.into();
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

    pub async fn run(&self, database: Arc<RikDataBase>) -> Result<()> {
        let client: RikControllerClient = RikControllerClient::connect().await?;

        let mut client_clone = client.clone();
        let database = database.clone();

        tokio::spawn(async move { client_clone.get_status_updates(database).await });

        self.listen_notification(client).await?;
        Ok(())
    }

    async fn handle_create(
        &self,
        instance: Instance,
        workload_def: WorkloadDefinition,
        client: &mut RikControllerClient,
    ) -> Result<()> {
        info!(
            "Ctrl to scheduler schedule instance workload_id : {:?}",
            &instance.workload_id
        );
        Ok(client
            .schedule_instance(WorkloadScheduling {
                workload_id: instance.workload_id,
                definition: serde_json::to_string(&workload_def).unwrap(),
                action: CRUD::Create as i32,
                instance_id: instance.id,
            })
            .await?)
    }

    async fn handle_delete(
        &self,
        instance: Instance,
        workload_def: WorkloadDefinition,
        client: &mut RikControllerClient,
    ) -> Result<()> {
        info!(
            "Ctrl to scheduler delete workload: {:?}",
            instance.workload_id
        );
        Ok(client
            .schedule_instance(WorkloadScheduling {
                workload_id: instance.workload_id,
                definition: serde_json::to_string(&workload_def).unwrap(),
                action: CRUD::Delete as i32,
                instance_id: instance.id,
            })
            .await?)
    }

    async fn listen_notification(&self, mut client: RikControllerClient) -> Result<()> {
        for notification in &self.internal_receiver {
            match notification.action {
                CRUD::Create => {
                    info!(
                        "Ctrl to scheduler schedule instance workload_id : {:?}",
                        notification.workload_id
                    );
                    let definition = notification.workload_definition.as_ref().unwrap().clone();
                    let instance: Instance = notification.into();
                    self.handle_create(instance, definition, &mut client)
                        .await?;
                }
                CRUD::Delete => {
                    info!(
                        "Ctrl to scheduler delete workload: {:?}",
                        notification.workload_id
                    );
                    let definition = notification.workload_definition.as_ref().unwrap().clone();
                    let instance: Instance = notification.into();
                    self.handle_delete(instance, definition, &mut client)
                        .await?;
                }
            }
        }
        Ok(())
    }
}
