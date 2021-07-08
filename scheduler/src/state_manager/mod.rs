mod lib;

use crate::state_manager::lib::{get_random_hash, int_to_resource_status, resource_status_to_int};
use definition::workload::WorkloadDefinition;
use log::{debug, error, info};
use proto::common::{InstanceMetric, ResourceStatus, WorkerMetric, WorkloadRequestKind};
use proto::worker::InstanceScheduling;
use rand::seq::IteratorRandom;
use rik_scheduler::{
    Event, SchedulerError, Worker, WorkerState, WorkloadChannelType, WorkloadRequest,
};
use std::collections::HashMap;
use std::fmt;
use std::slice::Iter;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;

#[derive(Debug)]
pub enum StateManagerEvent {
    Schedule(WorkloadRequest),
    Shutdown,
    InstanceUpdate(InstanceMetric),
    WorkerUpdate(String, WorkerMetric),
}

impl fmt::Display for StateManagerEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Eq, PartialEq)]
enum WorkloadStatus {
    PENDING,
    CREATING,
    DESTROYING,
    RUNNING,
}

pub struct StateManager {
    state: HashMap<String, Workload>,
    workers: Arc<Mutex<Vec<Worker>>>,
    manager_channel: Sender<Event>,
}

impl StateManager {
    pub async fn new(
        manager_channel: Sender<Event>,
        workers: Arc<Mutex<Vec<Worker>>>,
        mut receiver: Receiver<StateManagerEvent>,
    ) -> Result<(), SchedulerError> {
        debug!("Creating StateManager...");
        let mut state_manager = StateManager {
            // We define a mini capacity
            state: HashMap::with_capacity(20),
            manager_channel,
            workers,
        };
        debug!("StateManager receiver is ready");
        state_manager.run(receiver).await
    }

    async fn run(
        &mut self,
        mut receiver: Receiver<StateManagerEvent>,
    ) -> Result<(), SchedulerError> {
        while let Some(message) = receiver.recv().await {
            match message {
                StateManagerEvent::Shutdown => {
                    info!("Shutting down StateManager");
                    return Ok(());
                }
                StateManagerEvent::Schedule(workload) => self.process_schedule_request(workload),
                StateManagerEvent::InstanceUpdate(metrics) => {
                    self.manager_channel
                        .send(Event::InstanceMetric(
                            "scheduler".to_string(),
                            metrics.clone(),
                        ))
                        .await;
                    self.process_instance_update(metrics)
                }
                StateManagerEvent::WorkerUpdate(identifier, metrics) => {
                    self.process_metric_update(identifier, metrics)
                }
            };
            self.scan_workers();
            self.update_state().await;
        }
        Err(SchedulerError::StateManagerFailed)
    }

    async fn send(&self, data: Event) -> Result<(), SchedulerError> {
        self.manager_channel.send(data).await.map_err(|e| {
            error!(
                "Failed to send message from StateManager to Manager, error: {}",
                e
            );
            SchedulerError::ClientDisconnected
        })
    }

    fn scan_workers(&mut self) {
        let mut deactivated_workers = Vec::new();
        let mut state = self.workers.lock().unwrap();
        {
            for worker in state.iter_mut() {
                if worker.channel.is_closed() && worker.is_ready() {
                    worker.set_state(WorkerState::NotReady);
                    deactivated_workers.push(worker.id.clone());
                }
            }
        }

        // In the case we deactivated any worker, we want to reschedule the instances linked to that
        let mut instances_to_delete = Vec::new();
        let mut instances = self.state.iter_mut();
        {
            for (id, workload) in instances {
                for (instance_id, instance) in workload.instances.iter() {
                    if let Some(worker_id) = &instance.worker_id {
                        if deactivated_workers.contains(worker_id) {
                            instances_to_delete.push((id.clone(), instance_id.clone()));
                        }
                    }
                }
            }
        }

        for (workload_id, instance_id) in &instances_to_delete {
            if let Some(workload) = self.state.get_mut(workload_id) {
                workload.instances.remove(instance_id);
            }
        }
    }

    fn process_instance_update(&mut self, metrics: InstanceMetric) -> Result<(), SchedulerError> {
        debug!(
            "[process_instance_update] Instance {} and received {} status",
            metrics.instance_id, &metrics.status
        );
        let workload = self
            .state
            .iter_mut()
            .find(|(_, workload)| workload.instances.contains_key(&metrics.instance_id));

        if let Some((_, workload)) = workload {
            let status = int_to_resource_status(&metrics.status);
            if status == ResourceStatus::Terminated {
                debug!(
                    "Deleted instance {} on workload {}",
                    &metrics.instance_id, &workload.id
                );
                workload.instances.remove(&metrics.instance_id);
            } else {
                let instance = workload.instances.get_mut(&metrics.instance_id).unwrap();
                instance.status = int_to_resource_status(&metrics.status);
                info!(
                    "Instance {} updated status to {:#?}",
                    instance.id, &instance.status
                );
            }
        } else {
            error!(
                "Could not process instance {} update, as it does not exist",
                metrics.instance_id
            );
        }

        Ok(())
    }

    fn process_metric_update(
        &mut self,
        identifier: String,
        metrics: WorkerMetric,
    ) -> Result<(), SchedulerError> {
        error!("Metrics update is not implemented for now but are received",);

        let mut lock = self.workers.lock().unwrap();
        if let Some(worker) = lock.iter_mut().find(|worker| worker.id.eq(&identifier)) {
            if int_to_resource_status(&metrics.status) == ResourceStatus::Running {
                worker.set_state(WorkerState::Ready);
            } else {
                worker.set_state(WorkerState::NotReady);
            }
        } else {
            error!(
                "Received metrics for worker {} but could not find registration associated",
                identifier
            );
        }

        Ok(())
    }

    async fn update_state(&mut self) {
        if self.workers.lock().unwrap().len() == 0 {
            info!("State isn't updated as there is no worker available");
            return ();
        }

        let mut scheduled: Vec<(String, WorkloadInstance)> = Vec::new();

        // Well I'm sorry for this piece of code which isn't a art piece! Had some trouble with
        // ownership
        for (id, workload) in self.state.iter_mut() {
            let length_diff: i32 = (workload.replicas as i32 - (workload.instances.len() as i32));

            if length_diff > 0 {
                debug!(
                    "Divergence detected on {}, divergence length: {}",
                    workload.id, length_diff
                );
                for _ in 0..length_diff {
                    // Generate an instance ID, and ensure it is unique
                    let mut workload_id = get_random_hash(4).to_ascii_lowercase();
                    while workload.instances.contains_key(id) {
                        workload_id = get_random_hash(4).to_ascii_lowercase();
                    }
                    workload_id = format!("{}-{}", workload.definition.name.clone(), workload_id);

                    scheduled.push((
                        id.clone(),
                        WorkloadInstance::new(
                            workload_id.clone(),
                            ResourceStatus::Pending,
                            None,
                            workload.definition.clone(),
                        ),
                    ));
                }
            } else if length_diff < 0 {
                debug!(
                    "Divergence detected on {}, divergence length: {}",
                    workload.id, length_diff
                );
                let mut removed: Vec<String> = Vec::new();
                // As length_diff is negative, we need the opposite
                for _ in 0..(-length_diff) {
                    if let Some((id, instance)) = workload
                        .instances
                        .iter_mut()
                        .find(|(id, instance)| !removed.contains(id))
                    {
                        instance.status = ResourceStatus::Destroying;
                        debug!(
                            "WorkloadInstance {} went to {:#?}",
                            &instance.id, &instance.status
                        );

                        self.manager_channel
                            .send(Event::Schedule(
                                instance.worker_id.clone().unwrap(),
                                InstanceScheduling {
                                    instance_id: instance.id.clone(),
                                    action: WorkloadRequestKind::Destroy.into(),
                                    definition: serde_json::to_string(&instance.definition.clone())
                                        .unwrap(),
                                },
                            ))
                            .await;
                        self.manager_channel
                            .send(Event::InstanceMetric(
                                "scheduler".to_string(),
                                InstanceMetric {
                                    status: ResourceStatus::Destroying.into(),
                                    metrics: "".to_string(),
                                    instance_id: instance.id.clone(),
                                },
                            ))
                            .await;
                        removed.push(id.clone());
                    }
                }
            }
        }

        for (workload_id, mut instance) in scheduled.into_iter() {
            if let Some(worker_id) = self.get_eligible_worker() {
                self.manager_channel
                    .send(Event::Schedule(
                        worker_id.clone(),
                        InstanceScheduling {
                            instance_id: instance.id.clone(),
                            action: WorkloadRequestKind::Create as i32,
                            definition: serde_json::to_string(&instance.definition.clone())
                                .unwrap(),
                        },
                    ))
                    .await;
                self.manager_channel
                    .send(Event::InstanceMetric(
                        "scheduler".to_string(),
                        InstanceMetric {
                            status: ResourceStatus::Pending.into(),
                            metrics: format!("\"workload_id\": \"{}\"", workload_id.clone()),
                            instance_id: instance.id.clone(),
                        },
                    ))
                    .await;
                let state = self.state.get_mut(&workload_id).unwrap();
                {
                    instance.set_worker(Some(worker_id));
                    state.instances.insert(instance.id.clone(), instance);
                }
            } else {
                error!("Trying to schedule but cannot find any eligible worker");
            }
        }

        let mut to_be_deleted = Vec::new();
        for  key in self.state.keys().clone() {
            if let Some(workload) = self.state.get(key) {
                if workload.replicas == 0 && workload.instances.len() == 0 {
                    to_be_deleted.push(key.clone());
                }
            }
        }

        for workload in to_be_deleted {
            self.state.remove(&workload);
            debug!("Deleted workload {} from current state", workload);
        }
    }

    fn process_schedule_request(&mut self, request: WorkloadRequest) -> Result<(), SchedulerError> {
        debug!(
            "[process_schedule_request] Received workload id {}, action: {:#?}",
            request.workload_id, request.action
        );

        match request.action {
            WorkloadRequestKind::Create => self.action_create_workload(request),
            WorkloadRequestKind::Destroy => self.action_destroy_workload(request),
        }
    }

    fn action_create_workload(&mut self, request: WorkloadRequest) -> Result<(), SchedulerError> {
        if let Some(workload) = self.state.get(&request.workload_id) {
            if workload.status == ResourceStatus::Destroying {
                error!("Cannot double replicas while workload is being destroyed");
                return Err(SchedulerError::CannotDoubleReplicas);
            }

            let def_replicas = &workload.definition.replicas.unwrap_or(1);
            self.action_add_replicas(&request.workload_id, def_replicas)?;
        } else {
            let workload = Workload {
                id: request.workload_id,
                replicas: request.definition.replicas.unwrap_or(1),
                definition: request.definition,
                instances: HashMap::new(),
                status: ResourceStatus::Pending,
            };

            info!("[process_schedule_request] Received scheduling request for {}, with {:#?} replicas", workload.id, workload.definition.replicas);

            self.state.insert(workload.id.clone(), workload);
        }
        Ok(())
    }

    fn action_add_replicas(
        &mut self,
        workload_id: &String,
        replicas: &u16,
    ) -> Result<(), SchedulerError> {
        let mut workload = match self.state.get_mut(workload_id) {
            Some(wk) => Ok(wk),
            None => Err(SchedulerError::WorkloadDontExists(workload_id.clone())),
        }?;

        debug!(
            "[action_double_replicas] Adding replicas for {}, added {} to {}",
            workload_id, replicas, workload.replicas
        );

        workload.replicas += replicas;
        Ok(())
    }

    fn action_minus_replicas(
        &mut self,
        workload_id: &String,
        replicas: &u16,
    ) -> Result<(), SchedulerError> {
        let mut workload = match self.state.get_mut(workload_id) {
            Some(wk) => Ok(wk),
            None => Err(SchedulerError::WorkloadDontExists(workload_id.clone())),
        }?;
        debug!(
            "[action_double_replicas] Minus replicas for {}, removed {} to {}",
            workload_id, replicas, workload.replicas
        );

        workload.replicas -= replicas;

        Ok(())
    }

    fn action_destroy_workload(&mut self, request: WorkloadRequest) -> Result<(), SchedulerError> {
        let mut workload = self.state.get_mut(&request.workload_id);

        if workload.is_none() {
            error!(
                "Requested workload {} hasn't any instance available",
                request.workload_id
            );
            return Err(SchedulerError::WorkloadDontExists(request.workload_id));
        }

        let mut workload = workload.unwrap();

        if workload.status == ResourceStatus::Destroying {
            return Ok(());
        }

        let def_replicas = &workload.definition.replicas.unwrap_or(1);

        info!(
            "[process_schedule_request] Received destroy request for {}, with {:#?} replicas",
            workload.id, workload.definition.replicas
        );

        if workload.replicas > *def_replicas {
            self.action_minus_replicas(&request.workload_id, def_replicas)?;
        } else {
            info!("Workload {} is getting unscheduled", workload.id);
            workload.status = ResourceStatus::Destroying;
            workload.replicas = 0;
        }
        Ok(())
    }

    fn get_eligible_worker(&self) -> Option<String> {
        let workers = self.workers.lock().unwrap();
        {
            let workers = workers.iter().filter(|worker| worker.is_ready());
            if let Some(worker) = workers.choose(&mut rand::thread_rng()) {
                return Some(worker.id.clone());
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct Workload {
    /// The current number of replicas deployed for this workload
    replicas: u16,
    definition: WorkloadDefinition,
    instances: HashMap<String, WorkloadInstance>,
    status: ResourceStatus,
    id: String,
}

#[derive(Debug, Clone)]
pub struct WorkloadInstance {
    /// Part of the instance id that define the instance
    id: String,
    /// Current status of this instance
    status: ResourceStatus,
    /// Must be filled, the current id of the worker
    worker_id: Option<String>,
    /// Current definition for this workload
    definition: WorkloadDefinition,
}

impl WorkloadInstance {
    pub fn new(
        id: String,
        status: ResourceStatus,
        worker_id: Option<String>,
        definition: WorkloadDefinition,
    ) -> WorkloadInstance {
        WorkloadInstance {
            id,
            status,
            worker_id,
            definition,
        }
    }

    pub fn set_worker(&mut self, worker: Option<String>) {
        debug!(
            "WorkloadInstance {} was assigned to worker {}",
            self.id,
            worker.clone().unwrap_or("None".to_string())
        );
        self.worker_id = worker;
    }
}
