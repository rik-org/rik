use crate::config::Configuration;
use crate::emitters::metrics_emitter::MetricsEmitter;
use crate::structs::{Container, WorkloadDefinition};
use crate::traits::EventEmitter;
use cri::console::ConsoleSocket;
use cri::container::{CreateArgs, DeleteArgs, Runc};
use firepilot::microvm::{BootSource, Config, Drive, MicroVM};
use firepilot::Firecracker;
use node_metrics::metrics_manager::MetricsManager;
use oci::image_manager::ImageManager;
use proto::common::{InstanceMetric, WorkerMetric, WorkerRegistration, WorkerStatus};
use proto::worker::worker_client::WorkerClient;
use proto::worker::InstanceScheduling;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{stdout, Write};
use std::path::{Path, PathBuf};
use std::{fs, io, thread};
use std::time::Duration;
use tonic::{transport::Channel, Request, Streaming};
use curl::easy::Easy;
use lz4::{Decoder, EncoderBuilder};

#[derive(Debug)]
pub struct Riklet {
    hostname: String,
    client: WorkerClient<Channel>,
    stream: Streaming<InstanceScheduling>,
    image_manager: ImageManager,
    container_runtime: Runc,
    workloads: HashMap<String, Vec<Container>>,
}

impl Riklet {
    /// Display a banner
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

    /// Bootstrap a Riklet in order to run properly.
    pub async fn bootstrap() -> Result<Self, Box<dyn Error>> {
        // Get the hostname of the host to register
        let hostname = gethostname::gethostname().into_string().unwrap();

        // Display the banner, just for fun :D
        Riklet::banner();

        // Load the configuration
        let config = Configuration::load()?;

        // Connect to the master node scheduler
        let mut client = WorkerClient::connect(config.master_ip.clone()).await?;
        log::debug!("gRPC WorkerClient connected.");

        // Register this node to the master
        let request = Request::new(WorkerRegistration {
            hostname: hostname.clone(),
        });
        let stream = client.register(request).await?.into_inner();

        log::trace!("Registration success");

        // Initialize the container runtime
        let container_runtime = Runc::new(config.runner.clone())?;
        // Initialize the image manager
        let image_manager = ImageManager::new(config.manager.clone())?;

        Ok(Self {
            hostname,
            container_runtime,
            image_manager,
            client,
            stream,
            workloads: HashMap::<String, Vec<Container>>::new(),
        })
    }

    /// Handle a workload (eg CREATE, UPDATE, DELETE, READ)
    pub async fn handle_workload(
        &mut self,
        workload: &InstanceScheduling,
    ) -> Result<(), Box<dyn Error>> {
        match &workload.action {
            // Create
            0 => {
                self.create_workload(workload).await?;
            }
            // Delete
            1 => {
                self.delete_workload(workload).await?;
            }
            _ => {
                log::error!("Method not allowed")
            }
        }

        Ok(())
    }

    fn download_image(url: &String, file_path: &String) -> Result<(), Box<dyn Error>> {
        let mut easy = Easy::new();
        let mut dst = Vec::new();
        easy.url(&url).unwrap();
        let _redirect = easy.follow_location(true);

        {
            let mut transfer = easy.transfer();
            transfer.write_function(|data| {
                dst.extend_from_slice(data);
                Ok(data.len())
            }).unwrap();
            transfer.perform().unwrap();
        }

        {
            let mut file = File::create(&file_path)?;
            file.write_all(dst.as_slice())?;
        }

        Ok(())
    }

    fn decompress(source: &Path, destination: &Path) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(source)?;
        let mut decoder = Decoder::new(input_file)?;
        let mut output_file = File::create(destination)?;
        io::copy(&mut decoder, &mut output_file)?;
        Ok(())
    }

    async fn create_workload(
        &mut self,
        workload: &InstanceScheduling,
    ) -> Result<(), Box<dyn Error>> {
        let workload_definition: WorkloadDefinition =
            serde_json::from_str(&workload.definition[..]).unwrap();
        let instance_id: &String = &workload.instance_id;

        if workload_definition.kind == "Function" {
            log::info!("Function workload detected");

            let rootfs_url = workload_definition.spec.function.clone().unwrap().execution.rootfs.to_string();

            let download_directory = format!("/tmp/{}", &workload_definition.name);
            let mut file_path = format!("{}/rootfs.ext4", &download_directory);

            if !Path::new(&file_path).exists() {
                let mut lz4_path = format!("{}.lz4", &file_path);
                fs::create_dir(&download_directory)?;
                Self::download_image(&rootfs_url, &lz4_path)?;

                Self::decompress(Path::new(&lz4_path), Path::new(&file_path))?;
            }


            let firecracker = Firecracker::new(Some(firepilot::FirecrackerOptions {
                command: Some(PathBuf::from("/app/firecracker")),
                ..Default::default()
            }))
            .unwrap();

            let vm = MicroVM::from(Config {
                boot_source: BootSource {
                    kernel_image_path: PathBuf::from("/app/vmlinux.bin"),
                    boot_args: None,
                    initrd_path: None,
                },
                drives: vec![Drive {
                    drive_id: "rootfs".to_string(),
                    path_on_host: PathBuf::from(&file_path),
                    is_read_only: false,
                    is_root_device: true,
                }],
                network_interfaces: vec![],
            });
            thread::spawn(move || {
                firecracker.start(&vm).unwrap();
            });
        } else {
            log::info!("Container workload detected");

            let containers = workload_definition.get_containers(instance_id);

            // Inform the scheduler that the workload is creating
            self.send_status(5, instance_id).await;

            self.workloads
                .insert(instance_id.clone(), containers.clone());

            for container in containers {
                let id = container.id.unwrap();

                let image = &self.image_manager.pull(&container.image[..]).await?;

                // New console socket for the container
                let socket_path = PathBuf::from(format!("/tmp/{}", &id));
                let console_socket = ConsoleSocket::new(&socket_path)?;

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
                            log::error!("Receive PTY master error : {:?}", err)
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
                    .await?;

                log::info!("Started container {}", id);
            }
        }

        log::info!(
            "Workload '{}' successfully processed.",
            &workload.instance_id
        );

        // Inform the scheduler that the containers are running
        self.send_status(2, instance_id).await;

        Ok(())
    }

    async fn delete_workload(
        &mut self,
        workload: &InstanceScheduling,
    ) -> Result<(), Box<dyn Error>> {
        let instance_id = &workload.instance_id;
        let containers = self.workloads.get(&instance_id[..]).unwrap();

        for container in containers {
            self.container_runtime
                .delete(
                    &container.id.as_ref().unwrap()[..],
                    Some(&DeleteArgs { force: true }),
                )
                .await?;
            log::info!("Destroyed container {}", &container.id.as_ref().unwrap());
        }

        log::info!(
            "Workload '{}' successfully destroyed.",
            &workload.instance_id
        );

        // Inform the scheduler that the containers are running
        self.send_status(4, instance_id).await;

        Ok(())
    }

    async fn send_status(&self, status: i32, instance_id: &str) {
        MetricsEmitter::emit_event(
            self.client.clone(),
            vec![WorkerStatus {
                identifier: self.hostname.clone(),
                host_address: None,
                status: Some(proto::common::worker_status::Status::Instance(
                    InstanceMetric {
                        instance_id: instance_id.to_string().clone(),
                        status,
                        metrics: "".to_string(),
                    },
                )),
            }],
        )
        .await
        .unwrap();
    }

    /// Run the metrics updater
    fn start_metrics_updater(&self) {
        let client = self.client.clone();
        let hostname = self.hostname.clone();

        tokio::spawn(async move {
            let mut metrics_manager = MetricsManager::new();
            loop {
                let node_metric = metrics_manager.fetch();
                MetricsEmitter::emit_event(
                    client.clone(),
                    vec![WorkerStatus {
                        host_address: None,
                        identifier: hostname.clone(),
                        status: Some(proto::common::worker_status::Status::Worker(WorkerMetric {
                            status: 2,
                            metrics: node_metric.to_json().unwrap(),
                        })),
                    }],
                )
                .await
                .unwrap();
                tokio::time::sleep(Duration::from_millis(15000)).await;
            }
        });
    }

    /// Wait for workloads
    pub async fn accept(&mut self) -> Result<(), Box<dyn Error>> {
        log::info!("Riklet is running.");
        // Start the metrics updater
        self.start_metrics_updater();

        while let Some(workload) = &self.stream.message().await? {
            let _ = self.handle_workload(workload).await;
        }
        Ok(())
    }
}
