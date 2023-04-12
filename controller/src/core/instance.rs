use crate::api::ApiChannel;
use definition::workload::{Spec, WorkloadKind};
use definition::InstanceStatus;
use names::{Generator, Name};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
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

    pub spec: Spec,
}

impl From<ApiChannel> for Instance {
    fn from(value: ApiChannel) -> Self {
        let workload_definition = value.workload_definition.unwrap();
        Self {
            workload_id: value.workload_id.unwrap(),
            namespace: String::from("default"),
            kind: workload_definition.kind,
            id: value.instance_id.unwrap(),
            status: InstanceStatus::Pending,
            spec: workload_definition.spec,
        }
    }
}

impl Instance {
    pub fn new(workload_id: String, kind: WorkloadKind, id: Option<String>, spec: Spec) -> Self {
        Self {
            workload_id,
            namespace: String::from("default"),
            kind,
            id: id.unwrap_or_else(Self::generate_name),
            status: InstanceStatus::Pending,
            spec,
        }
    }

    pub fn generate_name() -> String {
        let mut random_name_generator = Generator::with_naming(Name::Numbered);
        random_name_generator.next().unwrap()
    }

    pub fn get_full_name(&self) -> String {
        format!("/instance/{}/{}/{}", self.kind, self.namespace, self.id)
    }
}
