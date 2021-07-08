use crate::metrics::Metrics;
use sysinfo::{System, SystemExt};

/// Struct managing node metrics
#[derive(Debug)]
pub struct MetricsManager {
    /// contains system's information
    pub system: System,
}

impl MetricsManager {
    /// Create MetricsManager
    pub fn new() -> MetricsManager {
        let sys = System::new_all();

        MetricsManager { system: sys }
    }

    /// Fetch system information
    pub fn fetch(&mut self) -> Metrics {
        self.system.refresh_all();
        Metrics::fetch(&self.system)
    }
}
