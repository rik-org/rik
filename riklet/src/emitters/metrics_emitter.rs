use crate::structs::EventEmitter;
use futures_util::stream;
use node_metrics::metrics_manager::MetricsManager;
use proto::common::{WorkerMetric, WorkerStatus};
use proto::worker::worker_client::WorkerClient;
use std::error::Error;
use std::time::Duration;
use tonic::transport::Channel;
use tonic::Request;
use tracing::{event, Level};

pub struct MetricsEmitter {
    manager: MetricsManager,
    identifier: String,
    client: WorkerClient<Channel>,
}

impl MetricsEmitter {
    pub fn new(identifier: String, client: WorkerClient<Channel>) -> Self {
        Self {
            manager: MetricsManager::new(),
            identifier,
            client,
        }
    }

    pub async fn emit_interval(&mut self, interval: u64) {
        loop {
            self.emit().await;
            tokio::time::sleep(Duration::from_millis(interval)).await;
        }
    }

    async fn emit(&mut self) {
        let node_metric = self.manager.fetch();
        let worker_status = WorkerStatus {
            host_address: None,
            identifier: self.identifier.clone(),
            status: Some(proto::common::worker_status::Status::Worker(WorkerMetric {
                status: 2,
                metrics: node_metric.to_json().unwrap(),
            })),
        };
        MetricsEmitter::emit_event(self.client.clone(), vec![worker_status])
            .await
            .unwrap_or_else(|err| event!(Level::ERROR, "Error while sending metrics : {:?}", err));
    }
}

#[async_trait::async_trait]
impl EventEmitter<Vec<WorkerStatus>, WorkerClient<Channel>> for MetricsEmitter {
    async fn emit_event(
        mut client: WorkerClient<Channel>,
        event: Vec<WorkerStatus>,
    ) -> std::result::Result<(), Box<dyn Error>> {
        // creating a new Request
        let request = Request::new(stream::iter(event));

        // sending request and waiting for response
        match client.send_status_updates(request).await {
            Ok(response) => {
                event!(Level::DEBUG, "Metrics was sent successfully.");
                response.into_inner()
            }
            Err(e) => event!(
                Level::ERROR,
                "An error occured when trying to send metrics: {:?}",
                e
            ),
        };

        Ok(())
    }
}
