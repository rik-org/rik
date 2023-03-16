use crate::{
    cli::{config::Configuration, function_config::FnConfiguration},
    iptables::{rule::Rule, Chain, Iptables, MutateIptables, Table},
    runtime::{NetworkError, RuntimeError, RuntimeManagerError},
    structs::WorkloadDefinition,
    IP_ALLOCATOR,
};
use async_trait::async_trait;
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
    process::Command,
    thread,
};
use tracing::{event, Level};

use super::{Network, NetworkDefinition, Runtime, RuntimeManager};

#[derive(Debug)]
struct FunctionRuntime {
    function_config: FnConfiguration,
    file_path: String,
    workload_definition: WorkloadDefinition,
    network_definition: Option<NetworkDefinition>,
}

#[async_trait]
impl Runtime for FunctionRuntime {
    async fn run(&mut self, network_definition: &NetworkDefinition) -> super::RuntimeResult<()> {
        event!(Level::INFO, "Function workload detected");

        event!(Level::INFO, "Define network");
        self.network_definition = Some(network_definition.clone());

        let firecracker = Firecracker::new(Some(firepilot::FirecrackerOptions {
            command: Some(self.function_config.firecracker_location.clone()),
            ..Default::default()
        }))
        .map_err(RuntimeError::Firecracker)?;

        event!(Level::DEBUG, "Creating a new MicroVM");
        let vm = MicroVM::from(Config {
            boot_source: BootSource {
                kernel_image_path: self.function_config.kernel_location.clone(),
                boot_args: Some(format!(
                    "console=ttyS0 reboot=k nomodules random.trust_cpu=on panic=1 pci=off tsc=reliable i8042.nokbd i8042.noaux ipv6.disable=1 quiet loglevel=0 ip={}::{}:{}::eth0:off",
                    network_definition.firecracker_ip, network_definition.tap_ip, network_definition.mask_long)
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
                guest_mac: Some("AA:FC:00:00:00:01".to_string()),
                host_dev_name: format!("rik-{}-tap", self.workload_definition.name),
            }],
        });

        event!(Level::DEBUG, "Starting the MicroVM");
        thread::spawn(move || -> super::RuntimeResult<()> {
            event!(Level::INFO, "Function started");
            firecracker.start(&vm).map_err(RuntimeError::Firecracker)?;
            Ok(())
        });

        /*
        let boot_args= format!("console=ttyS0 reboot=k nomodules random.trust_cpu=on panic=1 pci=off tsc=reliable i8042.nokbd i8042.noaux ipv6.disable=1 quiet loglevel=0 ip={firecracker_ip}::{tap_ip}:{MASK_LONG}::eth0:off");
        let firepilot = Firepilot::new(
            workload_definition,
            self.function_config,
            fs_definition.file_path,
        )
        .with_bootargs(boot_args.as_str())
        .with_guest_mac("AA:FC:00:00:00:01");
        thread::spawn(move || {
            event!(Level::INFO, "Function started");
            firepilot.start();
        });
        */
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
        easy.url(&url).map_err(RuntimeManagerError::CurlError)?;
        easy.follow_location(true)
            .map_err(RuntimeManagerError::CurlError)?;

        {
            let mut transfer = easy.transfer();
            transfer
                .write_function(|data| {
                    buffer.extend_from_slice(data);
                    Ok(data.len())
                })
                .map_err(RuntimeManagerError::CurlError)?;
            transfer.perform().map_err(RuntimeManagerError::CurlError)?;
        }

        let response_code = easy
            .response_code()
            .map_err(RuntimeManagerError::CurlError)?;
        if response_code != 200 {
            // return Err(format!("Response code from registry: {}", response_code).into());
        }

        {
            event!(Level::DEBUG, "Writing data to {}", file_path);
            let mut file = File::create(&file_path).map_err(RuntimeManagerError::IoError)?;
            file.write_all(buffer.as_slice())
                .map_err(RuntimeManagerError::IoError)?;
        }

        Ok(())
    }

    fn decompress(&self, source: &Path, destination: &Path) -> super::Result<()> {
        let input_file = File::open(source).map_err(RuntimeManagerError::IoError)?;
        let mut decoder = Decoder::new(input_file).map_err(RuntimeManagerError::IoError)?;
        let mut output_file = File::create(destination).map_err(RuntimeManagerError::IoError)?;
        io::copy(&mut decoder, &mut output_file).map_err(RuntimeManagerError::IoError)?;
        Ok(())
    }

    fn create_fs(&self, workload_definition: WorkloadDefinition) -> super::Result<String> {
        let rootfs_url = workload_definition.get_rootfs_url();

        let download_directory = format!("/tmp/{}", &workload_definition.name);
        let file_path = format!("{}/rootfs.ext4", &download_directory);
        let file_pathbuf = Path::new(&file_path);

        if !file_pathbuf.exists() {
            let lz4_path = format!("{}.lz4", &file_path);
            fs::create_dir(&download_directory).map_err(RuntimeManagerError::IoError)?;

            self.download_image(&rootfs_url, &lz4_path).map_err(|e| {
                event!(Level::ERROR, "Error while downloading image: {}", e);
                fs::remove_dir_all(&download_directory).expect("Error while removing directory");
                e
            })?;

            self.decompress(Path::new(&lz4_path), file_pathbuf)
                .map_err(|e| {
                    event!(Level::ERROR, "Error while decompressing image: {}", e);
                    fs::remove_dir_all(&download_directory)
                        .expect("Error while removing directory");
                    e
                })?;
        }
        Ok(file_path)
    }
}

impl RuntimeManager for FunctionRuntimeManager {
    fn create_network(&self, workload: InstanceScheduling) -> super::Result<Box<dyn Network>> {
        let workload_definition: WorkloadDefinition =
            serde_json::from_str(workload.definition.as_str())
                .map_err(RuntimeManagerError::JsonError)?;

        Ok(Box::new(FunctionNetwork {
            function_config: FnConfiguration::load(),
            workload_definition,
        }))
    }

    fn create_runtime(
        &self,
        workload: InstanceScheduling,
        _config: Configuration,
    ) -> super::Result<Box<dyn Runtime>> {
        event!(Level::INFO, "Function workload detected");
        let workload_definition: WorkloadDefinition =
            serde_json::from_str(workload.definition.as_str())
                .map_err(RuntimeManagerError::JsonError)?;

        Ok(Box::new(FunctionRuntime {
            function_config: FnConfiguration::load(),
            file_path: self.create_fs(workload_definition.clone())?,
            workload_definition,
            network_definition: None,
        }))
    }
}

struct FunctionNetwork {
    workload_definition: WorkloadDefinition,
    function_config: FnConfiguration,
}
impl Network for FunctionNetwork {
    fn init(&self) -> super::NetworkResult<NetworkDefinition> {
        println!("Function network initialized");
        let default_agent_port: u16 = 8080;

        // Get port to expose function
        let exposed_port = self.workload_definition.get_expected_port();

        // Alocate ip range for tap interface and firecracker micro VM
        let subnet = IP_ALLOCATOR
            .lock()
            .unwrap()
            .allocate_subnet()
            .ok_or("No more internal ip available")
            .map_err(|e| NetworkError::CommonNetworkError(e.to_string()))?;

        let tap_ip = subnet
            .nth(1)
            .ok_or("Fail get tap ip")
            .map_err(|e| NetworkError::CommonNetworkError(e.to_string()))?;
        let firecracker_ip = subnet
            .nth(2)
            .ok_or("Fail to get firecracker ip")
            .map_err(|e| NetworkError::CommonNetworkError(e.to_string()))?;
        let mask_long: &str = "255.255.255.252";

        let output = Command::new("/bin/sh")
            .arg(&self.function_config.script_path)
            .arg(&self.workload_definition.name)
            .arg(tap_ip.to_string())
            .output()
            .map_err(NetworkError::IoError)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.is_empty() {
                event!(Level::ERROR, "stderr: {}", stderr);
            }
            // return Err(stderr.into());
        }

        // Create a new IPTables object
        let mut ipt = Iptables::new(false).map_err(NetworkError::Iptables)?;

        // Port forward microvm on the host
        let rule = Rule {
            rule: format!(
                "-p tcp --dport {} -d {} -j DNAT --to-destination {}:{}",
                exposed_port, self.function_config.ifnet_ip, firecracker_ip, default_agent_port
            ),
            chain: Chain::Output,
            table: Table::Nat,
        };
        ipt.create(&rule).map_err(NetworkError::Iptables)?;

        // Allow NAT on the interface connected to the internet.
        let rule = Rule {
            rule: format!("-o {} -j MASQUERADE", self.function_config.ifnet),
            chain: Chain::PostRouting,
            table: Table::Nat,
        };
        ipt.create(&rule).map_err(NetworkError::Iptables)?;

        // Add the FORWARD rules to the filter table
        let rule = Rule {
            rule: "-m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT".to_string(),
            chain: Chain::Forward,
            table: Table::Filter,
        };
        ipt.create(&rule).map_err(NetworkError::Iptables)?;
        let rule = Rule {
            rule: format!(
                "-i rik-{}-tap -o {} -j ACCEPT",
                self.workload_definition.name, self.function_config.ifnet
            ),
            chain: Chain::Forward,
            table: Table::Filter,
        };
        ipt.create(&rule).map_err(NetworkError::Iptables)?;

        Ok(NetworkDefinition {
            mask_long: mask_long.to_string(),
            firecracker_ip,
            tap_ip,
        })
    }
}
