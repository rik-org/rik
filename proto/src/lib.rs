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

impl From<i32> for WorkloadRequestKind {
    fn from(w: i32) -> Self {
        match w {
            1 => WorkloadRequestKind::Destroy,
            _ => WorkloadRequestKind::Create,
        }
    }
}

impl From<i32> for ResourceStatus {
    fn from(w: i32) -> Self {
        match w {
            6 => ResourceStatus::Destroying,
            5 => ResourceStatus::Creating,
            4 => ResourceStatus::Terminated,
            3 => ResourceStatus::Failed,
            2 => ResourceStatus::Running,
            1 => ResourceStatus::Pending,
            _ => ResourceStatus::Unknown,
        }
    }
}

pub extern crate protobuf;
