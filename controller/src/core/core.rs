use crate::api::{ApiChannel, RikError, CRUD};
use crate::core::instance::Instance;
use crate::core::instance_repository::InstanceRepositoryImpl;
use crate::core::instance_service::InstanceServiceImpl;
use crate::core::{InstanceService, Listener};
use crate::database::RikDataBase;
use definition::workload::WorkloadDefinition;
use proto::common::worker_status::Status;
use proto::common::{InstanceMetric, WorkerMetric, WorkerStatus};
use std::net::SocketAddr;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::thread;
use tracing::{event, Level};

pub enum CoreInternalEvent {
    InstanceStatusUpdate(InstanceMetric),
    WorkerStatusUpdate {
        identifier: String,
        address: SocketAddr,
        metric: WorkerMetric,
    },
    Legacy(ApiChannel),
    CreateInstance(Instance, WorkloadDefinition),
    DeleteInstance(Instance, WorkloadDefinition),
}

/// Core is meant to be a mediator between controller components
/// It is responsible to forward properly actions and events to the right component
/// It is also responsible to handle legacy events
/// It is meant to be the only component that is aware of the legacy code
/// It is also responsible to handle properly transactions, and being able to rollback in
/// case of problem
pub struct Core {
    instance_service: InstanceServiceImpl,

    internal_receiver: Receiver<CoreInternalEvent>,
    internal_sender: Sender<CoreInternalEvent>,
}

impl Core {
    pub async fn new(database: Arc<RikDataBase>) -> Result<Core, RikError> {
        let (internal_sender, internal_receiver) = std::sync::mpsc::channel();

        let instance_repo = InstanceRepositoryImpl::new(database);
        let instance_svc = InstanceServiceImpl::new(instance_repo, internal_sender.clone()).await?;
        Ok(Core {
            instance_service: instance_svc,
            internal_receiver,
            internal_sender,
        })
    }

    pub fn get_sender(&self) -> Sender<CoreInternalEvent> {
        self.internal_sender.clone()
    }

    /// Forward messages taken from ApiChannel to CoreInternal channel
    /// Waiting to be removed when legacy code is removed
    pub fn run_legacy_listener(receiver: Receiver<ApiChannel>, sender: Sender<CoreInternalEvent>) {
        thread::spawn(move || loop {
            let message = receiver.recv().unwrap();
            sender.send(CoreInternalEvent::Legacy(message)).unwrap();
        });
    }

    /// Handle messages that are from Legacy events
    /// Waiting to be removed when legacy code is removed
    pub async fn handle_legacy_notification(&mut self, notification: ApiChannel) {
        let definition = notification.workload_definition.as_ref().unwrap().clone();
        match notification.action {
            CRUD::Create => {
                let instance: Instance = notification.into();
                self.internal_sender
                    .send(CoreInternalEvent::CreateInstance(instance, definition))
                    .unwrap();
            }
            CRUD::Delete => {
                let instance: Instance = notification.into();
                self.internal_sender
                    .send(CoreInternalEvent::DeleteInstance(instance, definition))
                    .unwrap();
            }
        };
    }

    pub async fn listen_notification(mut self) {
        self.instance_service.run_listen_thread();
        loop {
            let message = self.internal_receiver.recv().unwrap();
            match message {
                CoreInternalEvent::InstanceStatusUpdate(instance_metric) => self
                    .instance_service
                    .handle_instance_status_update(instance_metric),
                CoreInternalEvent::WorkerStatusUpdate {
                    identifier,
                    address,
                    metric,
                } => {
                    event!(
                        Level::INFO,
                        "Received worker status update from {} ({}) with {:#?}",
                        identifier,
                        address,
                        metric
                    )
                }
                CoreInternalEvent::Legacy(notification) => {
                    self.handle_legacy_notification(notification).await
                }
                CoreInternalEvent::CreateInstance(instance, definition) => {
                    self.instance_service
                        .create_instance(instance, definition)
                        .await
                        .unwrap();
                }
                CoreInternalEvent::DeleteInstance(instance, definition) => {
                    self.instance_service
                        .delete_instance(instance, definition)
                        .await
                        .unwrap();
                }
            }
        }
    }
}
