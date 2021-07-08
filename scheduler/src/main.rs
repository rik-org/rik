mod config_parser;
mod grpc;
mod state_manager;

use crate::config_parser::ConfigParser;
use crate::grpc::GRPCService;
use crate::state_manager::{StateManager, StateManagerEvent, Workload};
use env_logger::Env;
use log::{debug, error, info, warn};
use proto::common::worker_status::Status;
use proto::common::{InstanceMetric, WorkerStatus};
use proto::controller::controller_server::ControllerServer;
use proto::worker::worker_server::WorkerServer;
use proto::worker::InstanceScheduling;
use rand::seq::IteratorRandom;
use rik_scheduler::{Controller, SchedulerError, Worker, WorkerRegisterChannelType};
use rik_scheduler::{Event, WorkloadChannelType};
use std::collections::HashMap;
use std::default::Default;
use std::net::{SocketAddr, SocketAddrV4};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tonic::transport::Server;

#[derive(Debug)]
pub struct Manager {
    workers: Arc<Mutex<Vec<Worker>>>,
    channel: Receiver<Event>,
    controller: Option<Controller>,
    worker_increment: u8,
    state_manager: Sender<StateManagerEvent>,
}

impl Manager {
    async fn run(
        workers_listener: SocketAddrV4,
        controllers_listener: SocketAddrV4,
    ) -> Result<Manager, Box<dyn std::error::Error>> {
        let (sender, receiver) = channel::<Event>(1024);
        let (state_sender, receiver_sender) = channel::<StateManagerEvent>(1024);

        let mut instance = Manager {
            workers: Arc::new(Mutex::new(Vec::new())),
            channel: receiver,
            controller: None,
            worker_increment: 0,
            state_manager: state_sender,
        };
        instance.run_workers_listener(workers_listener, sender.clone());
        instance.run_controllers_listener(controllers_listener, sender.clone());
        let workers = instance.workers.clone();
        tokio::spawn(async move {
            if let Err(e) = StateManager::new(sender.clone(), workers, receiver_sender).await {
                error!("StateManager failed, reason: {}", e);
            }
        });

        let channel_listener = instance.listen();
        channel_listener.await?;
        Ok(instance)
    }

    fn run_workers_listener(&self, listener: SocketAddrV4, sender: Sender<Event>) {
        let server = WorkerServer::new(GRPCService::new(sender));
        tokio::spawn(async move {
            let server = Server::builder().add_service(server).serve(listener.into());

            info!("Worker gRPC listening on {}", listener);

            if let Err(e) = server.await {
                error!("{}", e);
            }
        });
    }

    fn run_controllers_listener(&self, listener: SocketAddrV4, sender: Sender<Event>) {
        let server = ControllerServer::new(GRPCService::new(sender));
        tokio::spawn(async move {
            let server = Server::builder().add_service(server).serve(listener.into());

            info!("Controller gRPC listening on {}", listener);

            if let Err(e) = server.await {
                error!("{}", e);
            }
        });
    }

    async fn listen(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while let Some(e) = self.channel.recv().await {
            match e {
                Event::Register(channel, addr, hostname) => {
                    if let Err(e) = self.register(channel.clone(), addr, hostname.clone()).await {
                        error!(
                            "Failed to register worker {} ({}), reason: {}",
                            hostname, addr, e
                        )
                    }
                }
                Event::ScheduleRequest(workload) => {
                    if let Err(e) = self
                        .state_manager
                        .send(StateManagerEvent::Schedule(workload))
                        .await
                    {
                        error!("Failed to communicate with StateManager, reason: {}", e);
                    }
                    if self.controller.is_none() {
                        warn!("Be aware there is no GetUpdates connected from a controller");
                    }
                }
                Event::Schedule(worker_id, instance) => {
                    if let Some(sender) = self.get_worker_sender(&worker_id) {
                        if let Err(e) = sender.send(Ok(instance)).await {
                            error!(
                                "Failed to communicate with worker {}, reason: {}",
                                worker_id, e
                            )
                        }
                    } else {
                        error!(
                            "Received Schedule event with an invalid worker {}",
                            worker_id
                        );
                    }
                }
                Event::Subscribe(channel, addr) => {
                    if let Some(controller) = &self.controller {
                        if controller.is_channel_closed() {
                            self.controller = Some(Controller::new(channel.clone(), addr));
                        } else {
                            error!("Can only have one controller at a time");
                        }
                    } else {
                        info!("A controller is now connected");
                        self.controller = Some(Controller::new(channel.clone(), addr));
                    }
                }
                Event::WorkerMetric(identifier, data) => {
                    let mut workers = self.workers.lock().unwrap();
                    if let Some(worker) =
                        workers.iter_mut().find(|worker| worker.id.eq(&*identifier))
                    {
                        debug!("Updated worker metrics for {}({})", identifier, worker.id);
                        match serde_json::from_str(&data.metrics) {
                            Ok(metric) => worker.set_metrics(metric),
                            Err(e) => warn!("Could not deserialize metrics, error: {}", e),
                        };
                    } else {
                        warn!(
                            "Received metrics for a unknown worker ({}), ignoring",
                            identifier
                        );
                    }
                }
                Event::InstanceMetric(identifier, metrics) => {
                    if let Some(controller) = &self.controller {
                        if let Err(e) = controller
                            .send(Ok(WorkerStatus {
                                identifier,
                                status: Some(Status::Instance(metrics)),
                            }))
                            .await
                        {
                            error!("Failed to send InstanceMetric to controller, reason: {}", e);
                        }
                    }
                }
                Event::InstanceMetricsUpdate(_, metrics) => {
                    self.state_manager
                        .send(StateManagerEvent::InstanceUpdate(metrics))
                        .await;
                }
                Event::WorkerMetricsUpdate(identifier, metrics) => {
                    self.state_manager
                        .send(StateManagerEvent::WorkerUpdate(identifier, metrics))
                        .await;
                }
                _ => unimplemented!("You think I'm not implemented ? Hold my beer"),
            }
        }
        Ok(())
    }

    fn get_next_id(&mut self) -> Result<u8, SchedulerError> {
        match self.worker_increment {
            u8::MAX => Err(SchedulerError::ClusterFull),
            _ => {
                self.worker_increment += 1;
                Ok(self.worker_increment)
            }
        }
    }

    fn get_worker_sender(&self, hostname: &str) -> Option<Sender<WorkerRegisterChannelType>> {
        if let Some(worker) = self
            .workers
            .lock()
            .unwrap()
            .iter_mut()
            .find(|worker| worker.id.eq(hostname))
        {
            return Some(worker.channel.clone());
        }

        None
    }

    async fn register(
        &mut self,
        channel: Sender<WorkerRegisterChannelType>,
        addr: SocketAddr,
        hostname: String,
    ) -> Result<(), SchedulerError> {
        let mut workers = self.workers.lock().unwrap();
        if let Some(worker) = workers.iter_mut().find(|worker| worker.id.eq(&*hostname)) {
            if !worker.channel.is_closed() {
                error!(
                    "New worker tried to register with an already taken hostname: {}",
                    hostname
                );
                channel
                    .send(Err(tonic::Status::already_exists(
                        "Worker with this hostname already exist",
                    )))
                    .await
                    .map_err(|_| SchedulerError::ClientDisconnected)?;
            } else {
                info!("Worker {} is back ready", hostname);
                worker.set_channel(channel);
            }
        } else {
            let worker = Worker::new(hostname, channel, addr);
            info!(
                "Worker {} is now registered, ip: {}",
                worker.id, worker.addr
            );
            workers.push(worker);
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ConfigParser::new()?;
    env_logger::Builder::from_env(Env::default().default_filter_or(&config.verbosity_level)).init();
    info!("Starting up...");
    let manager = Manager::run(config.workers_endpoint, config.controller_endpoint);
    manager.await?;
    Ok(())
}
