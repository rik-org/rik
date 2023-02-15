mod lib;

use crate::state_manager::lib::{get_random_hash, int_to_resource_status};
use definition::workload::WorkloadDefinition;
use log::{debug, error, info};
use proto::common::{InstanceMetric, ResourceStatus, WorkerMetric, WorkloadRequestKind};
use proto::worker::InstanceScheduling;
use rand::seq::IteratorRandom;
use scheduler::{Event, SchedulerError, Worker, WorkerState, WorkloadRequest};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;

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

pub struct StateManager {
    state: HashMap<String, Workload>,
    workers: Arc<Mutex<Vec<Worker>>>,
    manager_channel: Sender<Event>,
}

impl StateManager {
    pub fn new(manager_channel: Sender<Event>, workers: Arc<Mutex<Vec<Worker>>>) -> StateManager {
        debug!("Creating StateManager...");
        StateManager {
            // We define a mini capacity
            state: HashMap::with_capacity(20),
            manager_channel,
            workers,
        }
    }

    pub async fn run(
        &mut self,
        mut receiver: Receiver<StateManagerEvent>,
    ) -> Result<(), SchedulerError> {
        while let Some(message) = receiver.recv().await {
            let _ = match message {
                StateManagerEvent::Shutdown => {
                    info!("Shutting down StateManager");
                    return Ok(());
                }
                StateManagerEvent::Schedule(workload) => self.process_schedule_request(workload),
                StateManagerEvent::InstanceUpdate(metrics) => {
                    let _ = self
                        .manager_channel
                        .send(Event::InstanceMetric(
                            "scheduler".to_string(),
                            metrics.clone(),
                        ))
                        .await;
                    self.process_instance_update(metrics)
                }
                StateManagerEvent::WorkerUpdate(identifier, metrics) => {
                    self.process_metric_update(identifier, metrics).await
                }
            };
            self.scan_workers().await;
            self.update_state().await;
        }
        Err(SchedulerError::StateManagerFailed)
    }

    async fn scan_workers(&mut self) {
        let mut deactivated_workers = Vec::new();
        let mut state = self.workers.lock().await;
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
        let instances = self.state.iter_mut();
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

    async fn process_metric_update(
        &mut self,
        identifier: String,
        metrics: WorkerMetric,
    ) -> Result<(), SchedulerError> {
        let mut lock = self.workers.lock().await;
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
        let ready_workers = self.get_workers_ready().await;
        if ready_workers.len() == 0 {
            info!("State isn't updated as there is no worker available");
            return;
        }

        let mut workers = ready_workers.iter().cycle();
        // Scheduling of new instances
        for (id, workload) in self.state.iter_mut() {
            let pending_instances: Vec<&mut WorkloadInstance> = workload
                .instances
                .iter_mut()
                .filter_map(|(_, instance)| match instance.is_pending() {
                    true => Some(instance),
                    false => None,
                })
                .collect();

            for instance in pending_instances {
                let worker = workers.next().unwrap();

                instance.set_worker(Some(worker.clone()));
                instance.set_status(ResourceStatus::Creating);
                let _ = self
                    .manager_channel
                    .send(Event::Schedule(
                        worker.clone(),
                        InstanceScheduling {
                            instance_id: instance.id.clone(),
                            action: WorkloadRequestKind::Create as i32,
                            definition: serde_json::to_string(&instance.definition.clone())
                                .unwrap(),
                        },
                    ))
                    .await;
                let _ = self
                    .manager_channel
                    .send(Event::InstanceMetric(
                        "scheduler".to_string(),
                        InstanceMetric {
                            status: ResourceStatus::Creating.into(),
                            metrics: format!("\"workload_id\": \"{}\"", workload.id.clone()),
                            instance_id: instance.id.clone(),
                        },
                    ))
                    .await;
            }
        }

        let mut to_be_deleted = Vec::new();
        for key in self.state.keys().clone() {
            if let Some(workload) = self.state.get(key) {
                if workload.replicas == 0 && workload.instances.is_empty() {
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
        let instance = WorkloadInstance::new(
            request.instance_id.clone(),
            ResourceStatus::Pending,
            None,
            request.definition.clone(),
        );
        if let Some(workload) = self.state.get_mut(&request.workload_id) {
            if workload.status == ResourceStatus::Destroying {
                error!("Cannot double replicas while workload is being destroyed");
                return Err(SchedulerError::CannotDoubleReplicas);
            }

            workload.instances.insert(instance.id.clone(), instance);
            let def_replicas = &workload.definition.replicas.unwrap_or(1);
            self.action_add_replicas(&request.workload_id, def_replicas)?;
        } else {
            let workload = Workload {
                id: request.workload_id,
                replicas: request.definition.replicas.unwrap_or(1),
                definition: request.definition,
                instances: {
                    let mut map = HashMap::new();
                    map.insert(instance.id.clone(), instance);
                    map
                },
                status: ResourceStatus::Pending,
            };

            info!("[process_schedule_request] Received scheduling request for {}, with {:#?} replicas", workload.id, workload.definition.replicas);

            self.state.insert(workload.id.clone(), workload);
        }
        Ok(())
    }

    fn action_add_replicas(
        &mut self,
        workload_id: &str,
        replicas: &u16,
    ) -> Result<(), SchedulerError> {
        let workload = match self.state.get_mut(workload_id) {
            Some(wk) => Ok(wk),
            None => Err(SchedulerError::WorkloadNotExisting(workload_id.to_string())),
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
        workload_id: &str,
        replicas: &u16,
    ) -> Result<(), SchedulerError> {
        let workload = match self.state.get_mut(workload_id) {
            Some(wk) => Ok(wk),
            None => Err(SchedulerError::WorkloadNotExisting(workload_id.to_string())),
        }?;
        debug!(
            "[action_double_replicas] Minus replicas for {}, removed {} to {}",
            workload_id, replicas, workload.replicas
        );

        workload.replicas -= replicas;

        Ok(())
    }

    fn action_destroy_workload(&mut self, request: WorkloadRequest) -> Result<(), SchedulerError> {
        let workload = self.state.get_mut(&request.workload_id);

        if workload.is_none() {
            error!(
                "Requested workload {} hasn't any instance available",
                request.workload_id
            );
            return Err(SchedulerError::WorkloadNotExisting(request.workload_id));
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

    async fn get_eligible_worker(&self) -> Option<String> {
        let workers = self.workers.lock().await;
        {
            let workers = workers.iter().filter(|worker| worker.is_ready());
            if let Some(worker) = workers.choose(&mut rand::thread_rng()) {
                return Some(worker.id.clone());
            }
        }
        None
    }

    async fn get_workers_ready(&self) -> Vec<String> {
        let workers = self.workers.lock().await;
        workers
            .iter()
            .filter(|worker| worker.is_ready())
            .map(|worker| worker.id.clone())
            .collect()
    }
}

#[derive(Debug)]
pub struct Workload {
    /// Deployed replicas of the workload
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
            worker.clone().unwrap_or_else(|| "None".to_string())
        );
        self.worker_id = worker;
    }

    /// Determine whether the instance is running somewhere and has been properly running
    pub fn is_deployed(&self) -> bool {
        match self.status {
            ResourceStatus::Running => true,
            ResourceStatus::Creating => true,
            ResourceStatus::Destroying => true,
            _ => false,
        }
    }

    pub fn is_pending(&self) -> bool {
        return self.status == ResourceStatus::Pending;
    }

    pub fn set_status(&mut self, status: ResourceStatus) {
        self.status = status;
    }
}
