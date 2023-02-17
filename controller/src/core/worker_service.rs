use crate::api::RikError;
use crate::core::worker_repository::WorkerRepositoryImpl;
use crate::core::{WorkerRepository, WorkerService};
use proto::common::WorkerMetric;
use std::net::SocketAddr;

pub struct WorkerServiceImpl {
    repository: WorkerRepositoryImpl,
}

impl WorkerServiceImpl {
    pub fn new(repository: WorkerRepositoryImpl) -> WorkerServiceImpl {
        WorkerServiceImpl { repository }
    }
}

impl WorkerService for WorkerServiceImpl {
    fn handle_metric_update(
        &mut self,
        identifier: String,
        address: SocketAddr,
        metric: WorkerMetric,
    ) -> Result<(), RikError> {
        self.repository
            .register_worker(identifier, address.to_string())
    }
}
