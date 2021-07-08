use serde::{Deserialize, Serialize};
#[cfg(feature = "manager")]
use sysinfo::{DiskExt, ProcessorExt, System, SystemExt};

#[derive(Serialize, Deserialize, Debug)]
pub struct CpuMetrics {
    /// number of CPU
    pub total: u8,
    /// Pourcentage of total cpu usage
    pub free: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MemoryMetrics {
    /// Total memory (bytes)
    pub total: u64,
    /// Free memory (bytes)
    pub free: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DiskMetrics {
    pub disk_name: String,
    /// Total disk (bytes)
    pub total: u64,
    /// Free disk (bytes)
    pub free: u64,
}

/// Struct of node metrics
#[derive(Serialize, Deserialize, Debug)]
pub struct Metrics {
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
    pub disks: Vec<DiskMetrics>,
}

impl Metrics {
    #[cfg(feature = "manager")]
    pub fn fetch(sys: &System) -> Metrics {
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

    pub fn from_json(json: String) -> Result<Metrics, serde_json::Error> {
        serde_json::from_str(&json)
    }

    pub fn log(&self) {
        println!("{:?}", self)
    }
}
