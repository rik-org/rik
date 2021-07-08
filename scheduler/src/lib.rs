use definition::workload::WorkloadDefinition;
use log::{error, info};
use node_metrics::Metrics;
use proto::common::{InstanceMetric, WorkerMetric, WorkerStatus, WorkloadRequestKind};
use proto::controller::WorkloadScheduling;
use proto::worker::InstanceScheduling;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::net::SocketAddr;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::Sender;
use tonic::Status;

/// Define the structure of message send through the channel between
/// the manager and a worker
pub type WorkloadChannelType = Result<WorkloadScheduling, Status>;

pub type WorkerRegisterChannelType = Result<InstanceScheduling, tonic::Status>;

#[derive(Debug)]
pub enum Event {
    /// Workers register to the Scheduler so they can serve
    /// the cluster
    Register(Sender<WorkerRegisterChannelType>, SocketAddr, String),
    /// Controller can send workload, we use the verb Schedule to describe
    /// this event
    ScheduleRequest(WorkloadRequest),
    /// The StateManager uses this event to send a workload to a worker
    /// String is for the worker id
    Schedule(String, InstanceScheduling),
    /// This is meant for a controller subscription event
    /// Controller subscribe to the scheduler in order to get updqtes
    Subscribe(Sender<Result<WorkerStatus, Status>>, SocketAddr),
    /// Metrics received from workers to tell about themselves.
    /// The first string is the identifier, this event will send your metrics
    /// to the controller
    /// ```
    /// use proto::common::{WorkerMetric};
    /// let metrics = WorkerMetric {
    ///     status: 1,
    ///     metrics: "{metricA: 10, metricB: 100}".to_string()
    /// };
    /// ```
    WorkerMetric(String, WorkerMetric),
    /// Metrics received from workers to tell about themselves
    /// These metrics will be used inside the state manager
    WorkerMetricsUpdate(String, WorkerMetric),
    /// Metrics relative to a single instance of a workload, this event will
    /// send your metrics to the controller
    /// ```
    /// use proto::common::{InstanceMetric};
    /// let metrics = InstanceMetric {
    ///     status: 1,
    ///     metrics: "{metricA: 10, metricB: 100}".to_string(),
    ///     instance_id: "test"
    /// };
    /// ```
    InstanceMetric(String, InstanceMetric),
    /// Metrics received from workers to tell about themselves
    /// These metrics will be used inside the state manager
    InstanceMetricsUpdate(String, InstanceMetric),
}

#[derive(Debug)]
pub enum SchedulerError {
    /// Current max is 256 workers, given more workers, it returns
    /// a cluster full error
    ClusterFull,
    /// Worker registration process failed
    RegistrationFailed(String),
    /// gRPC client got disconnected
    ClientDisconnected,
    StateManagerFailed,
    /// In case we are destroying a workload and trying at the same time to re-schedule
    /// the workload, we prevent it
    CannotDoubleReplicas,
    /// In case we are ordering something but the workload doesn't exist in the
    /// memory
    WorkloadDontExists(String),
}

impl fmt::Display for SchedulerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for SchedulerError {}

#[derive(Debug)]
pub enum WorkerState {
    /// Worker is ready to receive workloads
    Ready,
    /// Worker is not / no more ready to receive workloads
    /// containers are relocated in case it switches from Ready state to non-ready
    NotReady,
}

impl fmt::Display for WorkerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkerState::Ready => write!(f, "Ready"),
            WorkerState::NotReady => write!(f, "Not Ready"),
        }
    }
}

#[derive(Debug)]
pub struct Controller {
    /// This channel is used to communicate between the manager
    /// and the controller
    channel: Sender<Result<WorkerStatus, Status>>,
    /// Remote addr of the controller
    addr: SocketAddr,
}

impl Controller {
    pub fn new(channel: Sender<Result<WorkerStatus, Status>>, addr: SocketAddr) -> Controller {
        Controller { channel, addr }
    }

    pub async fn send(
        &self,
        data: Result<WorkerStatus, Status>,
    ) -> Result<(), SendError<Result<WorkerStatus, Status>>> {
        self.channel.send(data).await.map_err(|e| {
            error!(
                "Failed to send message from Manager to Controller, error: {}",
                e
            );
            e
        })
    }

    pub fn is_channel_closed(&self) -> bool {
        self.channel.is_closed()
    }
}

#[derive(Debug)]
pub struct Worker {
    /// Worker hostname, must be unique
    pub id: String,
    /// This channel is used to communicate between the manager
    /// and the worker instance
    /// # Examples
    ///
    /// The following code is used in order to schedule an instance
    /// ```
    /// use rik_scheduler::{Worker, WorkloadChannelType};
    /// use proto::common::{Workload};
    /// use tokio::sync::mpsc::{channel, Receiver, Sender};
    /// use std::net::{SocketAddr, IpAddr, Ipv4Addr};
    /// let (sender, receiver) = channel::<WorkloadChannelType>(1024);
    /// let worker = Worker::new("debian-test".to_string(), sender, "127.0.0.1:8080".parse().unwrap());
    /// ```
    pub channel: Sender<WorkerRegisterChannelType>,
    /// Remote addr of the worker
    pub addr: SocketAddr,
    /// State of worker
    state: WorkerState,
    /// Most recent metric the worker has on its state
    metric: Option<Metrics>,
}

impl Worker {
    pub fn new(id: String, channel: Sender<WorkerRegisterChannelType>, addr: SocketAddr) -> Worker {
        Worker {
            id,
            channel,
            addr,
            state: WorkerState::NotReady,
            metric: None,
        }
    }

    pub fn set_channel(&mut self, sender: Sender<WorkerRegisterChannelType>) {
        self.channel = sender;
    }

    pub fn set_state(&mut self, state: WorkerState) {
        self.state = state;
        info!("Worker {} flipped to {} state", self.id, self.state);
    }

    pub fn get_state(&self) -> &WorkerState {
        &self.state
    }

    pub fn set_metrics(&mut self, metric: Metrics) {
        self.metric = Some(metric);
        self.update_state();
    }

    pub fn get_metrics(&self) -> &Option<Metrics> {
        &self.metric
    }

    fn update_state(&mut self) {
        match self.state {
            WorkerState::Ready => {
                if self.channel.is_closed() {
                    self.set_state(WorkerState::NotReady);
                }
            }
            WorkerState::NotReady => {
                if !self.channel.is_closed() {
                    self.set_state(WorkerState::Ready);
                }
            }
        }
    }

    pub async fn send(&self, data: InstanceScheduling) -> Result<(), SchedulerError> {
        self.channel.send(Ok(data)).await.map_err(|e| {
            error!("Failed to send message to remote worker, error: {}", e);
            SchedulerError::ClientDisconnected
        })
    }

    pub fn is_ready(&self) -> bool {
        matches!(self.state, WorkerState::Ready)
    }
}

#[tonic::async_trait]
pub trait Send<T> {
    async fn send(&self, data: T) -> Result<(), Status>;
}

#[derive(Debug)]
pub struct WorkloadRequest {
    pub workload_id: String,
    pub definition: WorkloadDefinition,
    pub action: WorkloadRequestKind,
}

impl WorkloadRequest {
    pub fn new(workload: WorkloadScheduling) -> Result<WorkloadRequest, serde_json::Error> {
        Ok(WorkloadRequest {
            workload_id: workload.workload_id,
            definition: serde_json::from_str(&workload.definition)?,
            action: match workload.action {
                1 => WorkloadRequestKind::Destroy,
                _ => WorkloadRequestKind::Create,
            },
        })
    }
}
