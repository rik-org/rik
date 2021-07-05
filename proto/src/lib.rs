use std::fmt;
use common::{WorkerMetric, InstanceMetric};

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

pub extern crate protobuf;
