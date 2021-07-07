use common::{InstanceMetric, ResourceStatus, WorkerMetric, WorkloadRequestKind};
use std::fmt;

pub mod common {
    tonic::include_proto!("common");
}

pub mod worker {
    tonic::include_proto!("worker");
}

pub mod controller {
    tonic::include_proto!("controller");
}

impl fmt::Display for WorkerMetric {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.status, self.metrics)
    }
}

impl fmt::Display for InstanceMetric {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.status, self.metrics)
    }
}

impl fmt::Display for ResourceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
impl fmt::Display for WorkloadRequestKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

pub extern crate protobuf;
