use crate::structs::WorkloadDefinition;
use std::thread;
use tracing::{event, Level};
pub trait Network {
    fn init(&self);
}
pub trait Runtime {
    fn run(&self);
}

pub trait RuntimeManager {
    fn create_network(&self) -> Box<dyn Network>;
    fn create_runtime(&self) -> Box<dyn Runtime>;

    fn create(&self) {
        let network = self.create_network();
        let runtime = self.create_runtime();

        network.init();
        runtime.run();
    }

    fn destroy(&self) {
        println!("Destroying runtime");
    }
}

struct FunctionNetwork {}
impl Network for FunctionNetwork {
    fn init(&self) {
        println!("Function network initialized");
    }
}
struct PodNetwork {}
impl Network for PodNetwork {
    fn init(&self) {
        println!("Pod network initialized");
    }
}
struct FunctionRuntime {}
impl Runtime for FunctionRuntime {
    fn run(&self) {
        println!("Function runtime running");
    }
}
struct PodRuntime {}
impl Runtime for PodRuntime {
    fn run(&self) {
        println!("Pod runtime running");
    }
}

struct FunctionRuntimeManager {}

impl RuntimeManager for FunctionRuntimeManager {
    fn create_network(&self) -> Box<dyn Network> {
        // let _network = Network::create_network(&workload_definition)?;

        Box::new(FunctionNetwork {})
    }

    fn create_runtime(&self) -> Box<dyn Runtime> {
        event!(Level::INFO, "Function workload detected");

        // let fs_definition = FsBuilder::new(&workload_definition);

        // let boot_args= format!("console=ttyS0 reboot=k nomodules random.trust_cpu=on panic=1 pci=off tsc=reliable i8042.nokbd i8042.noaux ipv6.disable=1 quiet loglevel=0 ip={firecracker_ip}::{tap_ip}:{MASK_LONG}::eth0:off");
        /*
        let firepilot = Firepilot::new(
            workload_definition,
            self.function_config,
            fs_definition.file_path,
        )
        .with_bootargs(boot_args.as_str())
        .with_guest_mac("AA:FC:00:00:00:01");
        */

        thread::spawn(move || {
            event!(Level::INFO, "Function started");
            // firepilot.start();
        });

        Box::new(FunctionRuntime {})
    }
}

struct PodRuntimeManager {}

impl RuntimeManager for PodRuntimeManager {
    fn create_network(&self) -> Box<dyn Network> {
        Box::new(PodNetwork {})
    }

    fn create_runtime(&self) -> Box<dyn Runtime> {
        Box::new(PodRuntime {})
    }
}

enum WorkloadKind {
    FUNCTION,
    POD,
}

impl Into<WorkloadKind> for String {
    fn into(self) -> WorkloadKind {
        match self.as_str() {
            "FUNCTION" => WorkloadKind::FUNCTION,
            "POD" => WorkloadKind::POD,
            _ => panic!("Unknown workload kind"),
        }
    }
}

pub struct RuntimeConfigurator {}
pub type DynamicRuntimeManager<'a> = &'a dyn RuntimeManager;
impl RuntimeConfigurator {
    pub fn create(workload_definition: &WorkloadDefinition) -> DynamicRuntimeManager {
        match workload_definition.kind.clone().into() {
            WorkloadKind::FUNCTION => &FunctionRuntimeManager {},
            WorkloadKind::POD => &PodRuntimeManager {},
        }
    }
}

/*
pub trait Runtime {
    fn run(&mut self);
}

pub struct RuntimeFactory {}

impl RuntimeFactory {
    pub fn build(workload_definition: &WorkloadDefinition) -> impl Runtime {
        match workload_definition.kind {
            WorkloadKind::FUNCTION => FunctionRuntime,
            WorkloadKind::POD => PodRuntime,
        }
    }
}

pub trait MainRuntime {
    type RuntimeImpl: Runtime;

    fn on_create(&self);
    fn on_destroy(&self);

    fn create(&self) {
        self.on_create();
        // Other stuff
    }

    fn destroy(&self) {
        self.on_destroy();
        // Other stuff
    }
}

pub struct FunctionRuntime {}

impl Runtime for FunctionRuntime {
    fn run(&mut self) {
        event!(Level::INFO, "Function workload detected");

        let fs_definition = FsBuilder::new(&workload_definition);
        let _network = Network::create_network(&workload_definition)?;

        let boot_args= format!("console=ttyS0 reboot=k nomodules random.trust_cpu=on panic=1 pci=off tsc=reliable i8042.nokbd i8042.noaux ipv6.disable=1 quiet loglevel=0 ip={firecracker_ip}::{tap_ip}:{MASK_LONG}::eth0:off");
        let firepilot = Firepilot::new(
            workload_definition,
            self.function_config,
            fs_definition.file_path,
        )
        .with_bootargs(boot_args.as_str())
        .with_guest_mac("AA:FC:00:00:00:01");

        thread::spawn(move || {
            firepilot.start();
        });
    }
}

pub struct PodRuntime {}

pub struct FsBuilder {
    pub file_path: String,
}

impl FsBuilder {
    pub fn new(workload_definition: &WorkloadDefinition) -> Self {
        workload_definition.get_rootfs_url();

        let download_directory = format!("/tmp/{}", &workload_definition.name);
        let file_path = format!("{}/rootfs.ext4", &download_directory);
        let file_pathbuf = Path::new(&file_path);

        if !file_pathbuf.exists() {
            let lz4_path = format!("{}.lz4", &file_path);
            fs::create_dir(&download_directory)?;

            Self::download_image(&rootfs_url, &lz4_path).map_err(|e| {
                event!(Level::ERROR, "Error while downloading image: {}", e);
                fs::remove_dir_all(&download_directory).expect("Error while removing directory"); // TODO error
                e
            })?;

            Self::decompress(Path::new(&lz4_path), file_pathbuf).map_err(|e| {
                event!(Level::ERROR, "Error while decompressing image: {}", e);
                fs::remove_dir_all(&download_directory).expect("Error while removing directory"); // TODO error
                e
            })?;
        }

        Self { file_path }
    }
}

*/
