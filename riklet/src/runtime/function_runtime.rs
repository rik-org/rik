use crate::runtime::Result;
use crate::{
    cli::function_config::FnConfiguration,
    network::tap::generate_mac_addr,
    runtime::{network::RuntimeNetwork, RuntimeError},
    structs::WorkloadDefinition,
};
use async_trait::async_trait;
use firepilot::builder::executor::FirecrackerExecutorBuilder;
use firepilot::builder::network_interface::NetworkInterfaceBuilder;
use firepilot::builder::{Configuration, Builder};
use firepilot::builder::drive::DriveBuilder;
use firepilot::builder::kernel::KernelBuilder;
use firepilot::machine::Machine;
use crate::cli::config::{Configuration as CliConfiguration};
use curl::easy::Easy;
use proto::worker::InstanceScheduling;
use std::{
    fs,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};
use tracing::{debug, event, Level};

use super::{network::function_network::FunctionRuntimeNetwork, Runtime, RuntimeManager};

const BOOT_ARGS_STATIC: &str = "console=ttyS0 reboot=k nomodules random.trust_cpu=on panic=1 pci=off tsc=reliable i8042.nokbd i8042.noaux ipv6.disable=1 quiet loglevel=0";

struct FunctionRuntime {
    id: String,
    /// Firecracker configuration
    function_config: FnConfiguration,
    /// Rootfs path on host
    file_path: String,
    network: FunctionRuntimeNetwork,
    /// microVM instance, expected to be None when nothing is running, and expected to
    /// to be fullfilled when the microVM is running
    machine: Option<Machine>
}

impl FunctionRuntime {
    fn generate_microvm_config(&self) -> Result<Configuration> {
        let kernel_args = format!(
            "{} ip={}::{}:{}::eth0:off",
            BOOT_ARGS_STATIC,
            self.network.host_ip,
            self.network.guest_ip,
            self.network.mask_long
        );
        let kernel_location = self
            .function_config
            .kernel_location
            .clone()
            .into_os_string()
            .into_string()
            .unwrap();
        let kernel = KernelBuilder::new()
            .with_kernel_image_path(kernel_location)
            .with_boot_args(kernel_args)
            .try_build()
            .map_err(RuntimeError::FirepilotConfiguration)?;   
        let drive = DriveBuilder::new()
            .with_drive_id("rootfs".to_string())
            .with_path_on_host(PathBuf::from(self.file_path.clone()))
            .as_root_device()
            .try_build()
            .map_err(RuntimeError::FirepilotConfiguration)?;
        let net_iface = NetworkInterfaceBuilder::new()
            .with_iface_id("eth0".to_string())
            .with_guest_mac(generate_mac_addr().to_string())
            .with_host_dev_name("riklet".to_string())
            // .with_host_dev_name(self.network.tap_name().map_err(RuntimeError::NetworkError)?)
            .try_build()
            .map_err(RuntimeError::FirepilotConfiguration)?;
        let executor = FirecrackerExecutorBuilder::new()
            .with_chroot("./srv".to_string())
            .with_exec_binary(self.function_config.firecracker_location.clone())
            .try_build()
            .map_err(RuntimeError::FirepilotConfiguration)?;

        let config = Configuration::new(self.id.clone())
            .with_kernel(kernel)
            .with_drive(drive)
            .with_interface(net_iface)
            .with_executor(executor);

        Ok(config)
    }
}

#[async_trait]
impl Runtime for FunctionRuntime {
    #[tracing::instrument(skip(self))]
    async fn up(&mut self) -> Result<()> {

        event!(Level::DEBUG, "Pre-boot configuration for microVM");
        let vm_config = self.generate_microvm_config()?;
        let mut machine = Machine::new();
        // Copy files and spawn the microVM socket, but it doesn't start the microVM
        machine.create(vm_config).await.map_err(RuntimeError::FirecrackerError)?;

        // self.network
        //     .init()
        //     .await
        //     .map_err(RuntimeError::NetworkError)?;

        // Start the microVM
        machine.start().await.map_err(RuntimeError::FirecrackerError)?;
        self.machine = Some(machine);
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn down(&self) -> Result<()> {
        debug!("Destroying function runtime vm");
        self.machine
            .as_ref()
            .unwrap()
            .stop()
            .await
            .map_err(RuntimeError::FirecrackerError)?;
        debug!("Destroying function runtime network");
        // self.network
        //     .destroy()
        //     .await
        //     .map_err(RuntimeError::NetworkError)
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
            return Err(RuntimeError::Error(format!(
                "Response code from registry: {}",
                response_code
            )));
        }

        {
            event!(Level::DEBUG, "Writing data to {}", file_path);
            let mut file = File::create(file_path).map_err(RuntimeError::IoError)?;
            file.write_all(buffer.as_slice())
                .map_err(RuntimeError::IoError)?;
        }

        Ok(())
    }

    // FIXME Commented because of a bug with the decompression - need to be fixed
    // fn decompress(&self, source: &Path, destination: &Path) -> super::Result<()> {
    //     let input_file = File::open(source).map_err(RuntimeError::IoError)?;
    //     let mut decoder = Decoder::new(input_file).map_err(RuntimeError::IoError)?;
    //     let mut output_file = File::create(destination).map_err(RuntimeError::IoError)?;
    //     io::copy(&mut decoder, &mut output_file).map_err(RuntimeError::IoError)?;
    //     Ok(())
    // }

    /// Download the rootfs image on the system if it does not exist
    fn create_fs(&self, workload_definition: &WorkloadDefinition) -> super::Result<String> {
        let rootfs_url = workload_definition
            .get_rootfs_url()
            .ok_or_else(|| RuntimeError::Error("Rootfs url not found".to_string()))?;

        let download_directory = format!("/tmp/{}", &workload_definition.name);
        let file_path = format!("{}/rootfs.ext4", &download_directory);
        let file_pathbuf = Path::new(&file_path);

        if !file_pathbuf.exists() {
            // FIXME Commented because of a bug with the decompression - need to be fixed
            // let lz4_path = format!("{}.lz4", &file_path);
            fs::create_dir(&download_directory).map_err(RuntimeError::IoError)?;

            self.download_image(&rootfs_url, &file_path).map_err(|e| {
                event!(Level::ERROR, "Error while downloading image: {}", e);
                fs::remove_dir_all(&download_directory).expect("Error while removing directory");
                e
            })?;
            // FIXME Commented because of a bug with the decompression - need to be fixed
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
        _config: CliConfiguration,
    ) -> super::Result<Box<dyn Runtime>> {
        event!(Level::DEBUG, "Function workload detected");
        let workload_definition: WorkloadDefinition =
            serde_json::from_str(workload.definition.as_str())
                .map_err(RuntimeError::ParsingError)?;

        Ok(Box::new(FunctionRuntime {
            function_config: FnConfiguration::load(),
            file_path: self.create_fs(&workload_definition)?,
            network: FunctionRuntimeNetwork::new(&workload).map_err(RuntimeError::NetworkError)?,
            machine: None,
            id: workload.instance_id
        }))
    }
}
