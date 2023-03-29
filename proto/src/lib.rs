use common::{worker_status::Status, InstanceMetric, ResourceStatus, WorkloadRequestKind};
use definition::InstanceStatus;
use std::ops::Deref;
pub mod common {
    tonic::include_proto!("common");
}

pub mod worker {
    tonic::include_proto!("worker");
}

pub mod controller {
    tonic::include_proto!("controller");
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

impl From<ResourceStatus> for InstanceStatus {
    fn from(value: ResourceStatus) -> Self {
        match value {
            ResourceStatus::Unknown => InstanceStatus::Unknown(String::from("")),
            ResourceStatus::Pending => InstanceStatus::Pending,
            ResourceStatus::Running => InstanceStatus::Running,
            ResourceStatus::Failed => InstanceStatus::Failed,
            ResourceStatus::Terminated => InstanceStatus::Terminated,
            ResourceStatus::Creating => InstanceStatus::Creating,
            ResourceStatus::Destroying => InstanceStatus::Destroying,
        }
    }
}

pub extern crate protobuf;

pub enum WorkloadAction {
    CREATE,
    DELETE,
}

pub struct WorkerStatus(pub common::WorkerStatus);
impl WorkerStatus {
    pub fn new(identifier: String, instance_id: String, status: InstanceStatus) -> Self {
        Self(common::WorkerStatus {
            identifier,
            host_address: None,
            status: Some(Status::Instance(InstanceMetric {
                instance_id,
                status: status.into(),
                metrics: "".to_string(),
            })),
        })
    }
}

impl Deref for WorkerStatus {
    type Target = common::WorkerStatus;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<i32> for WorkloadAction {
    fn from(value: i32) -> Self {
        match value {
            0 => WorkloadAction::CREATE,
            1 => WorkloadAction::DELETE,
            _ => panic!("Unknown workload action"),
        }
    }
}
