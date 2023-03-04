use crate::{
    cli::{config::Configuration, function_config::FnConfiguration},
    structs::WorkloadDefinition,
};
use async_trait::async_trait;
use cri::{
    console::ConsoleSocket,
    container::{CreateArgs, Runc},
};
use curl::easy::Easy;
use firepilot::{
    microvm::{BootSource, Config, Drive, MicroVM, NetworkInterface},
    Firecracker,
};
use lz4::Decoder;
use oci::image_manager::ImageManager;
use proto::worker::InstanceScheduling;
use shared::utils::ip_allocator::IpAllocator;
use std::net::Ipv4Addr;
use std::{
    fs,
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
    thread,
};
use tracing::{event, Level};

use super::{Network, NetworkDefinition, Runtime, RuntimeManager};

struct PodRuntime {
    image_manager: ImageManager,
    workload_definition: WorkloadDefinition,
    network_definition: Option<NetworkDefinition>,
    container_runtime: Runc,
    instance_id: String,
}

#[async_trait]
impl Runtime for PodRuntime {
    async fn run(&mut self, network_definition: &NetworkDefinition) {
        self.network_definition = Some(network_definition.clone());
        event!(Level::INFO, "Container workload detected");

        let containers = self.workload_definition.get_containers(&self.instance_id);

        // Inform the scheduler that the workload is creating
        // self.send_status(5, instance_id).await;

        // self.workloads
        //     .insert(instance_id.clone(), containers.clone());

        for container in containers {
            let id = container.id.unwrap();

            let image = &self.image_manager.pull(&container.image[..]).await.unwrap();

            // New console socket for the container
            let socket_path = PathBuf::from(format!("/tmp/{}", &id));
            let console_socket = ConsoleSocket::new(&socket_path).unwrap();

            tokio::spawn(async move {
                match console_socket
                    .get_listener()
                    .as_ref()
                    .unwrap()
                    .accept()
                    .await
                {
                    Ok((stream, _socket_addr)) => {
                        Box::leak(Box::new(stream));
                    }
                    Err(err) => {
                        event!(Level::ERROR, "Receive PTY master error : {:?}", err)
                    }
                }
            });
            self.container_runtime
                .run(
                    &id[..],
                    image.bundle.as_ref().unwrap(),
                    Some(&CreateArgs {
                        pid_file: None,
                        console_socket: Some(socket_path),
                        no_pivot: false,
                        no_new_keyring: false,
                        detach: true,
                    }),
                )
                .await
                .unwrap();

            event!(Level::INFO, "Started container {}", id);
        }
    }
}

pub struct PodRuntimeManager {}

impl RuntimeManager for PodRuntimeManager {
    fn create_network(
        &self,
        workload: InstanceScheduling,
        ip_allocator: IpAllocator,
    ) -> Box<dyn Network> {
        Box::new(PodNetwork {})
    }

    fn create_runtime(
        &self,
        workload: InstanceScheduling,
        config: Configuration,
    ) -> Box<dyn Runtime> {
        let workload_definition: WorkloadDefinition =
            serde_json::from_str(workload.definition.as_str()).unwrap();
        let instance_id: String = workload.instance_id;

        Box::new(PodRuntime {
            image_manager: ImageManager::new(config.manager.clone()).unwrap(),
            workload_definition,
            network_definition: None,
            container_runtime: Runc::new(config.runner.clone()).unwrap(),
            instance_id,
        })
    }
}

struct PodNetwork {}
impl Network for PodNetwork {
    fn init(&self) -> NetworkDefinition {
        println!("Pod network initialized");
        todo!()
    }
}
