use std::fmt::Display;

use common::{ResourceStatus, WorkloadRequestKind};
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Clone)]
pub enum InstanceStatus {
    Unknown(String),
    Pending,
    Running,
    Failed,
    Terminated,
    Creating,
    Destroying,
}

impl Display for InstanceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstanceStatus::Unknown(_) => write!(f, "Unknown"),
            InstanceStatus::Pending => write!(f, "Pending"),
            InstanceStatus::Running => write!(f, "Running"),
            InstanceStatus::Failed => write!(f, "Failed"),
            InstanceStatus::Terminated => write!(f, "Terminated"),
            InstanceStatus::Creating => write!(f, "Creating"),
            InstanceStatus::Destroying => write!(f, "Destroying"),
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

impl Into<i32> for InstanceStatus {
    fn into(self) -> i32 {
        match self {
            InstanceStatus::Unknown(_) => 0,
            InstanceStatus::Pending => 1,
            InstanceStatus::Running => 2,
            InstanceStatus::Failed => 3,
            InstanceStatus::Terminated => 4,
            InstanceStatus::Creating => 5,
            InstanceStatus::Destroying => 6,
        }
    }
}

impl From<i32> for InstanceStatus {
    fn from(value: i32) -> Self {
        match value {
            0 => InstanceStatus::Unknown(String::from("")),
            1 => InstanceStatus::Pending,
            2 => InstanceStatus::Running,
            3 => InstanceStatus::Failed,
            4 => InstanceStatus::Terminated,
            5 => InstanceStatus::Creating,
            6 => InstanceStatus::Destroying,
            _ => InstanceStatus::Unknown(String::from("")),
        }
    }
}

pub extern crate protobuf;
