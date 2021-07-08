use serde::{Deserialize, Serialize};

use sysinfo::{DiskExt, ProcessorExt, System, SystemExt};

#[derive(Serialize, Deserialize, Debug)]
struct CpuMetrics {
    /// number of CPU
    total: u8,
    /// Pourcentage of total cpu usage
    free: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct MemoryMetrics {
    /// Total memory (bytes)
    total: u64,
    /// Free memory (bytes)
    free: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct DiskMetrics {
    disk_name: String,
    /// Total disk (bytes)
    total: u64,
    /// Free disk (bytes)
    free: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metrics {
    cpu: CpuMetrics,
    memory: MemoryMetrics,
    disks: Vec<DiskMetrics>,
}

impl Metrics {
    pub fn new() -> Metrics {
        let mut sys = System::new_all();
        sys.refresh_all();

        // get cpu information
        let cpu_amount = sys.processors().len() as u8;
        let mut avg_cpu_usage = 0.0;
        for cpu in sys.processors() {
            let cpu_usage = cpu.cpu_usage();
            avg_cpu_usage += cpu_usage;
        }
        avg_cpu_usage /= cpu_amount as f32;

        // get memory information
        let memory_total = sys.total_memory();

        // get disk information
        let mut disks: Vec<DiskMetrics> = Vec::new();
        for disk in sys.disks() {
            let disk_name = match disk.name().to_str() {
                Some(name) => String::from(name),
                None => String::from("unknown"),
            };
            disks.push(DiskMetrics {
                disk_name: disk_name,
                total: disk.total_space(),
                free: disk.available_space(),
            })
        }

        Metrics {
            cpu: CpuMetrics {
                total: cpu_amount as u8,
                free: 100.0 - avg_cpu_usage,
            },
            memory: MemoryMetrics {
                total: 1024 * memory_total,
                free: 1024 * (memory_total - sys.used_memory()),
            },
            disks: disks,
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }

    pub fn log(&self) {
        println!("{:?}", self)
    }
}
