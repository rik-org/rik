use crate::api::types::instance::{Instance, InstanceStatus};
use crate::database::{RikDataBase, RikRepository};
use dotenv::dotenv;
use proto::common::worker_status::Status;
use proto::controller::controller_client::ControllerClient;
use proto::controller::WorkloadScheduling;
use rusqlite::Connection;
use std::sync::Arc;
use tracing::event;

#[derive(Clone)]
struct RikControllerClient {
    client: ControllerClient<tonic::transport::Channel>,
}

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

    /// gRPC Controller::ScheduleInstance route
    pub async fn schedule_instance(
        &mut self,
        instance: WorkloadScheduling,
    ) -> Result<(), tonic::Status> {
        let request = tonic::Request::new(instance);
        self.client.schedule_instance(request).await?;
        Ok(())
    }

    /// gRPC Controller::GetStatusUpdates route
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
