use names::Generator;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct InstanceDefinition {
    pub name: Option<String>,
    pub workload_id: String,
    pub replicas: Option<usize>,
}

#[allow(dead_code)]
impl InstanceDefinition {
    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    pub fn get_replicas(&mut self) -> usize {
        *self.replicas.get_or_insert(1)
    }

    pub fn get_name(&mut self) -> &mut String {
        let mut random_name_generator = Generator::default();
        self.name
            .get_or_insert(random_name_generator.next().unwrap())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Instance {
    pub id: usize,
    pub name: String,
    pub workload_id: usize,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InstanceStatus {
    // pub workload_id: String,
    pub status: String,
}
impl InstanceStatus {
    pub fn new(status: usize) -> InstanceStatus {
        let str_status = match status {
            0 => "Unknown".to_string(),
            1 => "Pending".to_string(),
            2 => "Running".to_string(),
            3 => "Failed".to_string(),
            4 => "Terminated".to_string(),
            5 => "Creating".to_string(),
            6 => "Destroying".to_string(),
            _ => "Creating".to_string(),
        };

        InstanceStatus {
            // workload_id: workload_id,
            status: str_status,
        }
    }
}
