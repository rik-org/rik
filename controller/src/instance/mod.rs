use crate::api::ApiChannel;
use definition::workload::WorkloadKind;
use names::{Generator, Name};
use proto::common::ResourceStatus;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum InstanceStatus {
    Unknown(String),
    Pending,
    Running,
    Failed,
    Terminated,
    Creating,
    Destroying,
}

#[derive(Serialize, Deserialize)]
pub struct Instance {
    /// Unique identifier of the workload
    pub workload_id: String,
    /// Namespace for the current instance, static to default for now
    pub namespace: String,
    /// Name composed with two words separated by a dash and
    /// finish with 4 digits
    pub id: String,

    pub kind: WorkloadKind,

    pub status: InstanceStatus,
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

impl From<ApiChannel> for Instance {
    fn from(value: ApiChannel) -> Self {
        Self {
            workload_id: value.workload_id.unwrap(),
            namespace: String::from("default"),
            kind: value.workload_definition.unwrap().kind,
            id: value.instance_id.unwrap(),
            status: InstanceStatus::Unknown(String::from("Generated with APIChannel event")),
        }
    }
}

impl Instance {
    pub fn new(workload_id: String, kind: WorkloadKind, id: Option<String>) -> Self {
        Self {
            workload_id,
            namespace: String::from("default"),
            kind,
            id: id.unwrap_or_else(Self::generate_name),
            status: InstanceStatus::Pending,
        }
    }

    fn generate_name() -> String {
        let mut random_name_generator = Generator::with_naming(Name::Numbered);
        random_name_generator.next().unwrap()
    }

    pub fn get_full_name(&self) -> String {
        format!("/instance/{}/{}/{}", self.kind, self.namespace, self.id)
    }

    pub fn repository_search_req(name: String) -> String {
        format!("/instance/%/default/{}", name)
    }
}
