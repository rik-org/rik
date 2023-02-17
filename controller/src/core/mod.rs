use crate::api::RikError;
use crate::core::core::CoreInternalEvent;
use crate::core::instance::Instance;
use async_trait::async_trait;
use backoff::ExponentialBackoff;
use definition::workload::WorkloadDefinition;
use proto::common::{InstanceMetric, WorkerMetric};
use std::future::Future;
use std::net::SocketAddr;
use std::time::Duration;
use tracing::{event, Level};

pub mod core;
pub mod instance;
mod instance_repository;
mod instance_service;
mod worker_repository;
mod worker_service;

trait Listener {
    fn run_listen_thread(&mut self);
}

#[async_trait]
trait InstanceService {
    async fn create_instance(
        &mut self,
        instance: Instance,
        workload_def: WorkloadDefinition,
    ) -> Result<(), RikError>;
    async fn delete_instance(
        &mut self,
        instance: Instance,
        workload_def: WorkloadDefinition,
    ) -> Result<(), RikError>;
    fn handle_instance_status_update(&mut self, instance_metric: InstanceMetric);
}

trait InstanceRepository {
    fn fetch_instance(&self, instance_id: String) -> Result<Instance, RikError>;
    fn register_instance(&self, instance: Instance) -> Result<(), RikError>;
}

trait WorkerService {
    fn handle_metric_update(
        &mut self,
        identifier: String,
        address: SocketAddr,
        metric: WorkerMetric,
    ) -> Result<(), RikError>;
}

trait WorkerRepository {
    fn fetch_worker_address(&self, worker_id: String) -> Result<String, RikError>;
    fn register_worker(&self, worker_id: String, address: String) -> Result<(), RikError>;
}

/// Create an exponential backoff function that retries a function until it succeeds or the timeout
/// is reached.
async fn with_backoff<F, T, E, BFuture>(f: F) -> Result<T, RikError>
where
    F: FnMut() -> BFuture,
    BFuture: Future<Output = Result<T, backoff::Error<E>>>,
    E: std::fmt::Display,
{
    let backoff = ExponentialBackoff {
        max_elapsed_time: Some(Duration::from_secs(60)),
        ..Default::default()
    };

    backoff::future::retry_notify(backoff, f, |_, next: Duration| {
        event!(
            Level::ERROR,
            "Backoff failed... retrying in {} seconds",
            next.as_secs()
        );
    })
    .await
    .map_err(|e| {
        RikError::InternalCommunicationError(format!("Could not connect to server: {}", e))
    })
}
