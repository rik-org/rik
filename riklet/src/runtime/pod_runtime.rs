use crate::{
    cli::config::Configuration,
    runtime::{network::RuntimeNetwork, RuntimeError},
    structs::WorkloadDefinition,
};
use async_trait::async_trait;
use cri::{
    console::ConsoleSocket,
    container::{CreateArgs, Runc},
};

use oci::image_manager::ImageManager;
use proto::worker::InstanceScheduling;
use std::path::PathBuf;
use tracing::{error, event, Level};

use super::{network::pod_network::PodRuntimeNetwork, Runtime, RuntimeManager};

#[derive(Debug)]
struct PodRuntime {
    image_manager: ImageManager,
    workload_definition: WorkloadDefinition,
    network: PodRuntimeNetwork,
    container_runtime: Runc,
    instance_id: String,
}

#[async_trait]
impl Runtime for PodRuntime {
    async fn up(&mut self) -> super::Result<()> {
        self.network
            .init()
            .await
            .map_err(RuntimeError::NetworkError)?;

        event!(Level::INFO, "Container workload detected");

        let containers = self.workload_definition.get_containers(&self.instance_id);

        for container in containers {
            if let Some(id) = container.id {
                let image = &self
                    .image_manager
                    .pull(&container.image[..])
                    .await
                    .map_err(RuntimeError::OciError)?;

                // New console socket for the container
                let socket_path = PathBuf::from(format!("/tmp/{}", &id));
                let console_socket =
                    ConsoleSocket::new(&socket_path).map_err(RuntimeError::CriError)?;

                tokio::spawn(async move {
                    if let Some(unix_listener) = console_socket.get_listener().as_ref() {
                        match unix_listener.accept().await {
                            Ok((stream, _socket_addr)) => {
                                Box::leak(Box::new(stream));
                            }
                            Err(err) => {
                                event!(Level::ERROR, "Receive PTY master error : {:?}", err)
                            }
                        }
                    }
                });
                self.container_runtime
                    .run(
                        &id[..],
                        image.bundle.as_ref().ok_or_else(|| {
                            RuntimeError::Error("Image bundle not found".to_string())
                        })?,
                        Some(&CreateArgs {
                            pid_file: None,
                            console_socket: Some(socket_path),
                            no_pivot: false,
                            no_new_keyring: false,
                            detach: true,
                        }),
                    )
                    .await
                    .map_err(RuntimeError::CriError)?;

                event!(Level::INFO, "Started container {}", id);
            }
        }
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(instance_id = %self.instance_id))]
    async fn down(&self) -> super::Result<()> {
        error!("Down not implemented for pod runtime");
        Ok(())
    }
}

pub struct PodRuntimeManager {}

impl RuntimeManager for PodRuntimeManager {
    fn create_runtime(
        &self,
        workload: InstanceScheduling,
        config: Configuration,
    ) -> super::Result<Box<dyn Runtime>> {
        let workload_definition: WorkloadDefinition =
            serde_json::from_str(workload.definition.as_str())
                .map_err(RuntimeError::ParsingError)?;
        let instance_id: String = workload.instance_id;

        Ok(Box::new(PodRuntime {
            image_manager: ImageManager::new(config.manager.clone())
                .map_err(RuntimeError::OciError)?,
            workload_definition,
            network: PodRuntimeNetwork::new(),
            container_runtime: Runc::new(config.runner).map_err(RuntimeError::CriError)?,
            instance_id,
        }))
    }
}
