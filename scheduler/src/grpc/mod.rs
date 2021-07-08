mod controller;
mod worker;

use crate::state_manager::Workload;
use definition::workload::{Spec, WorkloadDefinition};
use log::error;
use proto::controller::WorkloadScheduling;
use rik_scheduler::Send;
use rik_scheduler::{Event, WorkloadRequest};
use tokio::sync::mpsc::Sender;
use tonic::{Code, Status};

#[derive(Debug, Clone)]
pub struct GRPCService {
    /// Channel used in order to communicate with the main thread
    /// In the case the worker doesn't know its ID yet, put 0 in the first
    /// item of the tuple
    sender: Sender<Event>,
}

impl GRPCService {
    pub fn new(sender: Sender<Event>) -> GRPCService {
        GRPCService { sender }
    }
}

#[tonic::async_trait]
impl Send<Event> for GRPCService {
    async fn send(&self, data: Event) -> Result<(), Status> {
        self.sender.send(data).await.map_err(|e| {
            error!(
                "Failed to send message from gRPCService to Manager, error: {}",
                e
            );
            Status::new(
                Code::Unavailable,
                "We cannot process your request at this time",
            )
        })
    }
}
