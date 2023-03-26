use crate::{
    cli::{config::Configuration, function_config::FnConfiguration},
    network::tap::generate_mac_addr,
    runtime::{network::RuntimeNetwork, RuntimeError},
    structs::WorkloadDefinition,
};
use async_trait::async_trait;
use core::time;
use curl::easy::Easy;
use firepilot::{
    microvm::{BootSource, Config, Drive, MicroVM, NetworkInterface},
    Firecracker,
};
use lz4::Decoder;
use proto::worker::InstanceScheduling;
use std::{
    fs,
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
    thread,
};
use tracing::{error, event, Level};

use super::{network::function_network::FunctionRuntimeNetwork, Runtime, RuntimeManager};

struct FunctionRuntime {
    function_config: FnConfiguration,
    file_path: String,
    network: FunctionRuntimeNetwork,
    workload_definition: WorkloadDefinition,
}

#[async_trait]
impl Runtime for FunctionRuntime {
    async fn run(&mut self) -> super::Result<()> {
        self.network
            .init()
            .await
            .map_err(RuntimeError::NetworkError)?;

        event!(Level::INFO, "Define network");

        let firecracker = Firecracker::new(firepilot::FirecrackerOptions {
            command: Some(self.function_config.firecracker_location.clone()),
            ..Default::default()
        })
        .map_err(RuntimeError::FirecrackerError)?;

        event!(Level::DEBUG, "Creating a new MicroVM");
        let vm = MicroVM::from(Config {
            boot_source: BootSource {
                kernel_image_path: self.function_config.kernel_location.clone(),
                boot_args: Some(format!(
                    "console=ttyS0 reboot=k nomodules random.trust_cpu=on panic=1 pci=off tsc=reliable i8042.nokbd i8042.noaux ipv6.disable=1 quiet loglevel=0 ip={}::{}:{}::eth0:off",
                    self.network.firecracker_ip, self.network.tap_ip, self.network.mask_long)
                ),
                initrd_path: None,
            },
            drives: vec![Drive {
                drive_id: "rootfs".to_string(),
                path_on_host: PathBuf::from(self.file_path.clone()),
                is_read_only: false,
                is_root_device: true,
            }],
            network_interfaces: vec![NetworkInterface {
                iface_id: "eth0".to_string(),
                guest_mac: Some(generate_mac_addr().to_string()),
                host_dev_name: self.network.tap_name().map_err(RuntimeError::NetworkError)?,
            }],
        });

        event!(Level::DEBUG, "Starting the MicroVM");
        thread::spawn(move || {
            event!(Level::INFO, "Function started");
            let _stdout = firecracker.start(&vm).unwrap_or_else(|e| {
                error!("Error starting function: {}", e);
                String::from(format!("Error starting function: {}", e))
            });
        });

        let tap = self.network.tap.as_ref().unwrap();
        // small race condition between VM up & interface created
        let ten_millis = time::Duration::from_millis(10);
        thread::sleep(ten_millis);
        if let Err(e) = tap.set_link_up().await {
            error!("Could not bring iface {} up: {}", tap.iface_name(), e);
        }

        Ok(())
    }
}

pub struct FunctionRuntimeManager {}

impl FunctionRuntimeManager {
    fn download_image(&self, url: &String, file_path: &String) -> super::Result<()> {
        event!(
            Level::DEBUG,
            "Downloading image from {} to {}",
            url,
            file_path
        );

        let mut easy = Easy::new();
        let mut buffer = Vec::new();
        easy.url(url).map_err(RuntimeError::FetchingError)?;
        easy.follow_location(true)
            .map_err(RuntimeError::FetchingError)?;

        {
            let mut transfer = easy.transfer();
            transfer
                .write_function(|data| {
                    buffer.extend_from_slice(data);
                    Ok(data.len())
                })
                .map_err(RuntimeError::FetchingError)?;
            transfer.perform().map_err(RuntimeError::FetchingError)?;
        }

        let response_code = easy.response_code().map_err(RuntimeError::FetchingError)?;
        if response_code != 200 {
            // return Err(format!("Response code from registry: {}", response_code).into());
        }

        {
            event!(Level::DEBUG, "Writing data to {}", file_path);
            let mut file = File::create(file_path).map_err(RuntimeError::IoError)?;
            file.write_all(buffer.as_slice())
                .map_err(RuntimeError::IoError)?;
        }

        Ok(())
    }

    fn decompress(&self, source: &Path, destination: &Path) -> super::Result<()> {
        let input_file = File::open(source).map_err(RuntimeError::IoError)?;
        let mut decoder = Decoder::new(input_file).map_err(RuntimeError::IoError)?;
        let mut output_file = File::create(destination).map_err(RuntimeError::IoError)?;
        io::copy(&mut decoder, &mut output_file).map_err(RuntimeError::IoError)?;
        Ok(())
    }

    fn create_fs(&self, workload_definition: &WorkloadDefinition) -> super::Result<String> {
        let rootfs_url = workload_definition
            .get_rootfs_url()
            .ok_or_else(|| RuntimeError::Error("Rootfs url not found".to_string()))?;

        let download_directory = format!("/tmp/{}", &workload_definition.name);
        let file_path = format!("{}/rootfs.ext4", &download_directory);
        let file_pathbuf = Path::new(&file_path);

        if !file_pathbuf.exists() {
            // let lz4_path = format!("{}.lz4", &file_path);
            fs::create_dir(&download_directory).map_err(RuntimeError::IoError)?;

            self.download_image(&rootfs_url, &file_path).map_err(|e| {
                event!(Level::ERROR, "Error while downloading image: {}", e);
                fs::remove_dir_all(&download_directory).expect("Error while removing directory");
                e
            })?;

            // self.decompress(Path::new(&lz4_path), file_pathbuf)
            //     .map_err(|e| {
            //         event!(Level::ERROR, "Error while decompressing image: {}", e);
            //         fs::remove_dir_all(&download_directory)
            //             .expect("Error while removing directory");
            //         e
            //     })?;
        }
        Ok(file_path)
    }
}

impl RuntimeManager for FunctionRuntimeManager {
    fn create_runtime(
        &self,
        workload: InstanceScheduling,
        _config: Configuration,
    ) -> super::Result<Box<dyn Runtime>> {
        event!(Level::INFO, "Function workload detected");
        let workload_definition: WorkloadDefinition =
            serde_json::from_str(workload.definition.as_str())
                .map_err(RuntimeError::ParsingError)?;

        Ok(Box::new(FunctionRuntime {
            function_config: FnConfiguration::load(),
            file_path: self.create_fs(&workload_definition)?,
            network: FunctionRuntimeNetwork::new(&workload).map_err(RuntimeError::NetworkError)?,
            workload_definition,
        }))
    }
}
