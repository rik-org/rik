use crate::api::types::instance::InstanceStatus;
use crate::api::{ ApiChannel, CRUD };
use crate::database::RikDataBase;
use crate::database::RikRepository;
use anyhow::{ Result, Context };
use dotenv::dotenv;
use log::{ info, error };
use proto::common::worker_status::Status;
use proto::controller::controller_client::ControllerClient;
use proto::controller::WorkloadScheduling;
use rusqlite::Connection;
use std::sync::mpsc::{ Receiver, Sender };
use std::sync::Arc;
use uuid::Uuid;
use colored::Colorize;

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
        let client = ControllerClient::connect(scheduler_url).await.context(
            "Fail to connect to scheduler"
        )?;
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
        while let Some(status) = stream.message().await? {
            println!("Received status update request {:?}", status);
            if let Some(status) = status.status {
                let instance_id = match status.clone() {
                    Status::Instance(instance_metric) => Some(instance_metric.instance_id),
                    Status::Worker(_worker_metric) => None,
                };
                let instance_status = match status {
                    Status::Instance(instance_metric) => Some(instance_metric),
                    Status::Worker(_worker_metric) => None,
                };
                if let (Some(instance_id), Some(instance_status)) = (instance_id, instance_status) {
                    let id: String;
                    if
                        let Ok(previous_instance) = RikRepository::check_duplicate_name(
                            &connection,
                            &format!("/instance/default/{}", instance_id)
                        )
                    {
                        id = previous_instance.id;
                    } else {
                        id = Uuid::new_v4().to_string();
                    }
                    let instance_status = InstanceStatus::new(instance_status.status as usize);

                    let name = format!("/instance/default/{}", instance_id);
                    let value = serde_json::to_string(&instance_status)?;
                    RikRepository::upsert(
                        &connection,
                        &id,
                        &name,
                        &value,
                        &"/instance".to_string()
                    )?;
                }
            }
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
        internal_receiver: Receiver<ApiChannel>
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

    async fn listen_notification(&self, mut client: RikControllerClient) -> Result<()> {
        for notification in &self.internal_receiver {
            match notification.action {
                CRUD::Create => {
                    // Create instance
                    // Send workload to sheduler
                    info!(
                        "Ctrl to scheduler schedule instance workload_id : {:?}",
                        notification.workload_id
                    );
                    if let Some(workload_id) = notification.workload_id {
                        if let Some(workload_definition) = notification.workload_definition {
                            client.schedule_instance(WorkloadScheduling {
                                workload_id,
                                definition: serde_json::to_string(&workload_definition).unwrap(),
                                action: CRUD::Create as i32,
                            }).await?;
                        }
                    }
                }
                CRUD::Delete => {
                    // Delete instance
                    // Send instruction to sheduler
                    info!("Ctrl to scheduler delete workload: {:?}", notification.workload_id);

                    if let Some(workload_id) = notification.workload_id {
                        if let Some(workload_definition) = notification.workload_definition {
                            client.schedule_instance(WorkloadScheduling {
                                workload_id,
                                definition: serde_json::to_string(&workload_definition)?,
                                action: CRUD::Delete as i32,
                            }).await?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}