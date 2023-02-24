use crate::cli::config::Configuration;
use crate::cli::function_config::FnConfiguration;
use crate::emitters::metrics_emitter::MetricsEmitter;
use crate::structs::{Container, WorkloadDefinition};
use crate::traits::EventEmitter;
use cri::console::ConsoleSocket;
use cri::container::{CreateArgs, DeleteArgs, Runc};
use curl::easy::Easy;
use firepilot::microvm::{BootSource, Config, Drive, MicroVM, NetworkInterface};
use firepilot::Firecracker;
use ipnetwork::Ipv4Network;
use lz4::Decoder;
use node_metrics::metrics_manager::MetricsManager;
use oci::image_manager::ImageManager;
use proto::common::{InstanceMetric, WorkerMetric, WorkerRegistration, WorkerStatus};
use proto::worker::worker_client::WorkerClient;
use proto::worker::InstanceScheduling;
use shared::utils::ip_allocator::IpAllocator;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use std::{fs, io, thread};
use tonic::{transport::Channel, Request, Streaming};
use tracing::{event, Level};

// const TAP_SCRIPT_DEFAULT_LOCATION: &str = "/app/setup-host-tap.sh";
const MASK_LONG: &str = "255.255.255.252";
const DEFAULT_AGENT_PORT: u16 = 8080;

#[derive(Debug)]
pub struct Riklet {
    hostname: String,
    client: WorkerClient<Channel>,
    stream: Streaming<InstanceScheduling>,
    image_manager: ImageManager,
    container_runtime: Runc,
    workloads: HashMap<String, Vec<Container>>,
    ip_allocator: IpAllocator,
    function_config: FnConfiguration,
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
        event!(Level::DEBUG, "Riklet bootstraping process started.");
        // Get the hostname of the host to register
        let hostname = gethostname::gethostname().into_string().unwrap();

        // Display the banner, just for fun :D
        Riklet::banner();

        // Load the configuration
        let config = Configuration::load()?;

        // load the function runtime configuration
        let function_config = FnConfiguration::load();

        // Connect to the master node scheduler
        let mut client = WorkerClient::connect(config.master_ip.clone()).await?;
        event!(Level::DEBUG, "gRPC WorkerClient connected.");

        event!(Level::DEBUG, "Node's registration to the master");
        let request = Request::new(WorkerRegistration {
            hostname: hostname.clone(),
        });
        let stream = client.register(request).await?.into_inner();

        event!(Level::DEBUG, "Container runtime initialization");
        let container_runtime = Runc::new(config.runner.clone())?;
        event!(Level::DEBUG, "Image manager initialization");
        let image_manager = ImageManager::new(config.manager.clone())?;

        // Initialize the ip allocator
        let network = Ipv4Network::new(Ipv4Addr::new(192, 168, 1, 0), 24).unwrap();
        let ip_allocator = IpAllocator::new(network);

        Ok(Self {
            hostname,
            container_runtime,
            image_manager,
            client,
            stream,
            workloads: HashMap::<String, Vec<Container>>::new(),
            ip_allocator,
            function_config,
        })
    }

    /// Handle a workload (eg CREATE, UPDATE, DELETE, READ)
    pub async fn handle_workload(
        &mut self,
        workload: &InstanceScheduling,
    ) -> Result<(), Box<dyn Error>> {
        event!(Level::DEBUG, "Handling workload");
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
                event!(Level::ERROR, "Method not allowed")
            }
        }

        Ok(())
    }

    fn download_image(url: &String, file_path: &String) -> Result<(), Box<dyn Error>> {
        event!(
            Level::DEBUG,
            "Downloading image from {} to {}",
            url,
            file_path
        );

        let mut easy = Easy::new();
        let mut buffer = Vec::new();
        easy.url(&url)?;
        easy.follow_location(true)?;

        {
            let mut transfer = easy.transfer();
            transfer.write_function(|data| {
                buffer.extend_from_slice(data);
                Ok(data.len())
            })?;
            transfer.perform()?;
        }

        let response_code = easy.response_code()?;
        if response_code != 200 {
            return Err(format!("Response code from registry: {}", response_code).into());
        }

        {
            event!(Level::DEBUG, "Writing data to {}", file_path);
            let mut file = File::create(&file_path)?;
            file.write_all(buffer.as_slice())?;
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
        event!(Level::DEBUG, "Creating workload");
        let workload_definition: WorkloadDefinition =
            serde_json::from_str(&workload.definition[..]).unwrap();
        let instance_id: &String = &workload.instance_id;

        if workload_definition.kind == "Function" {
            event!(Level::INFO, "Function workload detected");

            let rootfs_url = workload_definition
                .spec
                .function
                .clone()
                .unwrap()
                .execution
                .rootfs
                .to_string();

            let download_directory = format!("/tmp/{}", &workload_definition.name);
            let file_path = format!("{}/rootfs.ext4", &download_directory);

            let file_pathbuf = Path::new(&file_path);
            if !file_pathbuf.exists() {
                let lz4_path = format!("{}.lz4", &file_path);
                fs::create_dir(&download_directory)?;

                Self::download_image(&rootfs_url, &lz4_path).map_err(|e| {
                    event!(Level::ERROR, "Error while downloading image: {}", e);
                    fs::remove_dir_all(&download_directory)
                        .expect("Error while removing directory");
                    e
                })?;

                Self::decompress(Path::new(&lz4_path), file_pathbuf).map_err(|e| {
                    event!(Level::ERROR, "Error while decompressing image: {}", e);
                    fs::remove_dir_all(&download_directory)
                        .expect("Error while removing directory");
                    e
                })?;
            }

            // Get port to expose function
            let exposed_port = workload_definition
                .spec
                .function
                .map(|f| f.exposure.map(|e| e.port))
                .flatten()
                .ok_or(())
                .unwrap();

            // Alocate ip range for tap interface and firecracker micro VM
            let subnet = self
                .ip_allocator
                .allocate_subnet()
                .ok_or("No more internal ip available")?;

            let tap_ip = subnet.nth(1).ok_or("Fail get tap ip")?;
            let firecracker_ip = subnet.nth(2).ok_or("Fail to get firecracker ip")?;

            let output = Command::new("/bin/sh")
                .arg(&self.function_config.script_path)
                .arg(&workload_definition.name)
                .arg(tap_ip.to_string())
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if !stderr.is_empty() {
                    event!(Level::ERROR, "stderr: {}", stderr);
                }
                return Err(stderr.into());
            }

            // Create a new IPTables object
            let ipt = iptables::new(false).unwrap();

            // Port forward microvm on the host
            ipt.append(
                "nat",
                "OUTPUT",
                &format!(
                    "-p tcp --dport {} -d {} -j DNAT --to-destination {}:{}",
                    exposed_port, self.function_config.ifnet_ip, firecracker_ip, DEFAULT_AGENT_PORT
                ),
            )
            .unwrap();

            // Allow NAT on the interface connected to the internet.
            ipt.append(
                "nat",
                "POSTROUTING",
                &format!("-o {} -j MASQUERADE", self.function_config.ifnet),
            )
            .unwrap();

            // Add the FORWARD rules to the filter table
            ipt.append_unique(
                "filter",
                "FORWARD",
                &"-m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT",
            )
            .unwrap();
            ipt.append(
                "filter",
                "FORWARD",
                &format!(
                    "-i rik-{}-tap -o {} -j ACCEPT",
                    workload_definition.name, self.function_config.ifnet
                ),
            )
            .unwrap();

            let firecracker = Firecracker::new(Some(firepilot::FirecrackerOptions {
                command: Some(PathBuf::from(&self.function_config.firecracker_location)),
                ..Default::default()
            }))
            .unwrap();

            event!(Level::DEBUG, "Creating a new MicroVM");
            let vm = MicroVM::from(Config {
                boot_source: BootSource {
                    kernel_image_path: PathBuf::from(
                        &self.function_config.kernel_location,
                    ),
                    boot_args: Some(format!(
                        "console=ttyS0 reboot=k nomodules random.trust_cpu=on panic=1 pci=off tsc=reliable i8042.nokbd i8042.noaux ipv6.disable=1 quiet loglevel=0 ip={firecracker_ip}::{tap_ip}:{MASK_LONG}::eth0:off"
                    )),
                    initrd_path: None,
                },
                drives: vec![Drive {
                    drive_id: "rootfs".to_string(),
                    path_on_host: PathBuf::from(&file_path),
                    is_read_only: false,
                    is_root_device: true,
                }],
                network_interfaces: vec![NetworkInterface {
                    iface_id: "eth0".to_string(),
                    guest_mac: Some("AA:FC:00:00:00:01".to_string()),
                    host_dev_name: format!("rik-{}-tap", workload_definition.name),
                }],
            });

            event!(Level::DEBUG, "Starting the MicroVM");
            thread::spawn(move || {
                firecracker.start(&vm).unwrap();
            });

            event!(
                Level::INFO,
                "Function '{}' scheduled and available at {}:{}",
                workload_definition.name,
                firecracker_ip,
                DEFAULT_AGENT_PORT
            )
        } else {
            event!(Level::INFO, "Container workload detected");

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
                    .await?;

                event!(Level::INFO, "Started container {}", id);
            }
        }

        event!(
            Level::INFO,
            "Workload '{}' successfully processed.",
            &workload.instance_id
        );

        event!(
            Level::DEBUG,
            "Informing the scheduler that the containers are running"
        );
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
            event!(
                Level::INFO,
                "Destroying container {}",
                &container.id.as_ref().unwrap()
            );
            self.container_runtime
                .delete(
                    &container.id.as_ref().unwrap()[..],
                    Some(&DeleteArgs { force: true }),
                )
                .await
                .unwrap_or_else(|err| {
                    event!(Level::ERROR, "Error while destroying container : {:?}", err)
                });
        }

        event!(
            Level::INFO,
            "Workload '{}' successfully destroyed.",
            &workload.instance_id
        );

        // Inform the scheduler that the containers are running
        self.send_status(4, instance_id).await;

        Ok(())
    }

    async fn send_status(&self, status: i32, instance_id: &str) {
        event!(Level::DEBUG, "Sending status : {}", status);
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
        .unwrap_or_else(|err| event!(Level::ERROR, "Error while sending status : {:?}", err));
    }

    /// Run the metrics updater
    fn start_metrics_updater(&self) {
        event!(Level::INFO, "Starting metrics updater");
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
                .unwrap_or_else(|err| {
                    event!(Level::ERROR, "Error while sending metrics : {:?}", err)
                });
                tokio::time::sleep(Duration::from_millis(15000)).await;
            }
        });
    }

    /// Wait for workloads
    pub async fn accept(&mut self) -> Result<(), Box<dyn Error>> {
        event!(Level::INFO, "Riklet is running.");
        // Start the metrics updater
        self.start_metrics_updater();

        while let Some(workload) = &self.stream.message().await? {
            let _ = self.handle_workload(workload).await;
        }
        Ok(())
    }
}
