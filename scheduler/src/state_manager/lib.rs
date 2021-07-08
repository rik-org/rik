use proto::common::ResourceStatus;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

pub fn get_random_hash(size: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size)
        .map(char::from)
        .collect()
}

pub fn resource_status_to_int(status: &ResourceStatus) -> i32 {
    match status {
        ResourceStatus::Destroying => 6,
        ResourceStatus::Creating => 5,
        ResourceStatus::Terminated => 4,
        ResourceStatus::Failed => 3,
        ResourceStatus::Running => 2,
        ResourceStatus::Pending => 1,
        ResourceStatus::Unknown => 0,
    }
}

pub fn int_to_resource_status(status: &i32) -> ResourceStatus {
    match status {
        6 => ResourceStatus::Destroying,
        5 => ResourceStatus::Creating,
        4 => ResourceStatus::Terminated,
        3 => ResourceStatus::Failed,
        2 => ResourceStatus::Running,
        1 => ResourceStatus::Pending,
        0 | _ => ResourceStatus::Unknown,
    }
}
