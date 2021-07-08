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

pub fn int_to_resource_status(status: &i32) -> ResourceStatus {
    match status {
        6 => ResourceStatus::Destroying,
        5 => ResourceStatus::Creating,
        4 => ResourceStatus::Terminated,
        3 => ResourceStatus::Failed,
        2 => ResourceStatus::Running,
        1 => ResourceStatus::Pending,
        _ => ResourceStatus::Unknown,
    }
}
