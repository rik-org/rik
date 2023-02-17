use crate::traits::EventEmitter;
use futures_util::stream;
use proto::common::WorkerStatus;
use proto::worker::worker_client::WorkerClient;
use std::error::Error;
use tonic::transport::Channel;
use tonic::Request;
use tracing::{event, Level};

pub struct MetricsEmitter;

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
