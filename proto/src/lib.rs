use common::{ResourceStatus, WorkloadRequestKind};

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

pub extern crate protobuf;
