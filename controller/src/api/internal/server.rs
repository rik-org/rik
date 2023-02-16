use crate::api::{ApiChannel, CRUD};
use crate::database::RikDataBase;
use crate::instance::Instance;
use definition::workload::WorkloadDefinition;
use proto::controller::WorkloadScheduling;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tracing::event;

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
