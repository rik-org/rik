use crate::cli::config::Configuration;
use crate::emitters::metrics_emitter::MetricsEmitter;
use crate::iptables::rule::Rule;
use crate::iptables::{Chain, Iptables, MutateIptables, Table};
use crate::network::net::{Net, NetworkInterfaceConfig};
use crate::runtime::{DynamicRuntimeManager, Runtime, RuntimeConfigurator};
use crate::structs::{Container, WorkloadDefinition};
use crate::traits::EventEmitter;
use cri::container::Runc;
use ipnetwork::Ipv4Network;
use oci::image_manager::ImageManager;
use proto::common::{InstanceMetric, WorkerRegistration, WorkerStatus};
use proto::worker::worker_client::WorkerClient;
use proto::worker::InstanceScheduling;
use proto::InstanceStatus;
use shared::utils::ip_allocator::IpAllocator;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};

use std::time::Duration;
use std::{fs, io, thread};
use std::ops::Deref;
use thiserror::Error;
use tonic::{transport::Channel, Request, Streaming};
use tracing::{debug, event, Level};

// const TAP_SCRIPT_DEFAULT_LOCATION: &str = "/app/setup-host-tap.sh";
const MASK_LONG: &str = "255.255.255.252";
const DEFAULT_AGENT_PORT: u16 = 8080;

const METRICS_UPDATER_INTERVAL: u64 = 15 * 1000;

#[derive(Error, Debug)]
pub enum RikletError {}
type Result<T> = std::result::Result<T, RikletError>;

enum WorkloadAction {
    CREATE,
    DELETE,
}

struct RikletWorkerStatus(WorkerStatus);
impl RikletWorkerStatus {
    fn new(identifier: String, instance_id: String, status: InstanceStatus) -> Self {
        Self(WorkerStatus {
            identifier,
            host_address: None,
            status: Some(proto::common::worker_status::Status::Instance(
                InstanceMetric {
                    instance_id,
                    status: status.into(),
                    metrics: "".to_string(),
                },
            )),
        })
    }
}

impl Deref for RikletWorkerStatus {
    type Target = WorkerStatus;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Into<WorkloadAction> for i32 {
    fn into(self) -> WorkloadAction {
        match self {
            1 => WorkloadAction::CREATE,
            2 => WorkloadAction::DELETE,
            _ => panic!("Unknown workload action"),
        }
    }
}

#[derive(Debug)]
pub struct Riklet {
    config: Configuration,
    hostname: String,
    client: WorkerClient<Channel>,
    stream: Streaming<InstanceScheduling>,
    runtimes: HashMap<String, Box<dyn Runtime>>,
    ip_allocator: IpAllocator,
    // function_config: FnConfiguration,
}

impl Riklet {
    async fn handle_workload(&mut self, workload: &InstanceScheduling) {
        event!(Level::DEBUG, "Handling workload");
        let workload_definition: WorkloadDefinition =
            serde_json::from_str(workload.definition.as_str()).unwrap();

        let dynamic_runtime_manager: DynamicRuntimeManager =
            RuntimeConfigurator::create(&workload_definition);

        match &workload.action.into() {
            WorkloadAction::CREATE => {
                self.create_workload(workload, dynamic_runtime_manager)
                    .await
            }
            WorkloadAction::DELETE => {
                self.delete_workload(workload, dynamic_runtime_manager)
                    .await
            }
            _ => event!(Level::ERROR, "Method not allowed"),
        }
    }

    async fn create_workload(
        &mut self,
        workload: &InstanceScheduling,
        dynamic_runtime_manager: DynamicRuntimeManager<'_>,
    ) {
        event!(Level::DEBUG, "Creating workload");
        let instance_id: &String = &workload.instance_id;
        let runtime = dynamic_runtime_manager
            .create(workload, self.ip_allocator.clone(), self.config.clone())
            .await;

        self.runtimes.insert(instance_id.clone(), runtime);

        self.send_status(InstanceStatus::Running, instance_id).await;
    }

    async fn delete_workload(
        &mut self,
        workload: &InstanceScheduling,
        runtime: DynamicRuntimeManager<'_>,
    ) {
        event!(Level::DEBUG, "Destroying workload");
        let instance_id: &String = &workload.instance_id;
        // Destroy the runtime
        // TODO

        self.runtimes.remove(instance_id);

        self.send_status(InstanceStatus::Terminated, instance_id)
            .await;

        runtime.destroy();
    }

    fn banner() {
        println!(
            r#"
        ______ _____ _   __ _      _____ _____
        | ___ \_   _| | / /| |    |  ___|_   _|
        | |_/ / | | | |/ / | |    | |__   | |
        |    /  | | |    \ | |    |  __|  | |
        | |\ \ _| |_| |\  \| |____| |___  | |
        \_| \_|\___/\_| \_/\_____/\____/  \_/
        "#
        );
    }

    async fn send_status(&self, status: InstanceStatus, instance_id: &str) {
        event!(Level::DEBUG, "Sending status : {}", status);

        let status =
            RikletWorkerStatus::new(self.hostname.clone(), instance_id.to_string(), status);

        MetricsEmitter::emit_event(self.client.clone(), vec![status.0])
            .await
            .unwrap_or_else(|err| event!(Level::ERROR, "Error while sending status : {:?}", err));
    }

    pub async fn run(&mut self) {
        event!(Level::INFO, "Riklet is running.");
        self.start_metrics_updater();

        while let Some(workload) = self.stream.message().await.unwrap() {
            self.handle_workload(&workload).await;
        }
    }

    fn start_metrics_updater(&self) {
        event!(Level::INFO, "Starting metrics updater");
        let client = self.client.clone();
        let hostname = self.hostname.clone();

        tokio::spawn(async move {
            let mut metrics_emitter = MetricsEmitter::new(hostname.clone(), client.clone());
            metrics_emitter
                .emit_interval(METRICS_UPDATER_INTERVAL)
                .await;
        });
    }

    pub async fn new() -> Result<Self> {
        event!(Level::DEBUG, "Riklet bootstraping process started.");
        Riklet::banner();
        let hostname = gethostname::gethostname().into_string().unwrap();

        let config = Configuration::load().unwrap();
        // let function_config = FnConfiguration::load();

        let mut client = WorkerClient::connect(config.master_ip.clone())
            .await
            .unwrap();
        event!(Level::DEBUG, "gRPC WorkerClient connected.");

        event!(Level::DEBUG, "Node's registration to the master");
        let request = Request::new(WorkerRegistration {
            hostname: hostname.clone(),
        });
        let stream = client.register(request).await.unwrap().into_inner();

        // TODO Network
        let network = Ipv4Network::new(Ipv4Addr::new(192, 168, 1, 0), 24).unwrap();
        let ip_allocator = IpAllocator::new(network);

        Ok(Self {
            hostname,
            client,
            stream,
            runtimes: HashMap::<String, Box<dyn Runtime>>::new(),
            ip_allocator,
            config,
        })
    }
