use crate::{cli::config::Configuration, runtime::RuntimeError, structs::WorkloadDefinition};
use async_trait::async_trait;
use cri::{
    console::ConsoleSocket,
    container::{CreateArgs, Runc},
};

use oci::image_manager::ImageManager;
use proto::worker::InstanceScheduling;
use std::path::PathBuf;
use tracing::{event, Level};

use super::{Network, NetworkDefinition, Runtime, RuntimeManager, RuntimeManagerError};

#[derive(Debug)]
struct PodRuntime {
    image_manager: ImageManager,
    workload_definition: WorkloadDefinition,
    network_definition: Option<NetworkDefinition>,
    container_runtime: Runc,
    instance_id: String,
}

#[async_trait]
impl Runtime for PodRuntime {
    async fn run(&mut self, network_definition: &NetworkDefinition) -> super::RuntimeResult<()> {
        self.network_definition = Some(network_definition.clone());
        event!(Level::INFO, "Container workload detected");

        let containers = self.workload_definition.get_containers(&self.instance_id);

        // Inform the scheduler that the workload is creating
        // self.send_status(5, instance_id).await;

        // self.workloads
        //     .insert(instance_id.clone(), containers.clone());

        for container in containers {
            let id = container.id.unwrap(); // TODO Some / None

            let image = &self
                .image_manager
                .pull(&container.image[..])
                .await
                .map_err(RuntimeError::OCI)?;

            // New console socket for the container
            let socket_path = PathBuf::from(format!("/tmp/{}", &id));
            let console_socket = ConsoleSocket::new(&socket_path).map_err(RuntimeError::CRI)?;

            tokio::spawn(async move {
                match console_socket
                    .get_listener()
                    .as_ref()
                    .unwrap() // TODO Some / None
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
                    image.bundle.as_ref().unwrap(), // TODO Some / None
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
        Ok(())
    }
}

pub struct PodRuntimeManager {}

impl RuntimeManager for PodRuntimeManager {
    fn create_network(&self, workload: InstanceScheduling) -> super::Result<Box<dyn Network>> {
        Ok(Box::new(PodNetwork {}))
    }

    fn create_runtime(
        &self,
        workload: InstanceScheduling,
        config: Configuration,
    ) -> super::Result<Box<dyn Runtime>> {
        let workload_definition: WorkloadDefinition =
            serde_json::from_str(workload.definition.as_str()).unwrap(); // TODO Some / None
        let instance_id: String = workload.instance_id;

        Ok(Box::new(PodRuntime {
            image_manager: ImageManager::new(config.manager.clone())
                .map_err(RuntimeManagerError::OCI)?,
            workload_definition,
            network_definition: None,
            container_runtime: Runc::new(config.runner.clone())
                .map_err(RuntimeManagerError::CRI)?,
            instance_id,
        }))
    }
}

struct PodNetwork {}
impl Network for PodNetwork {
    fn init(&self) -> super::NetworkResult<NetworkDefinition> {
        println!("Pod network initialized");
        todo!()
    }
}
