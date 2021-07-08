use serde::{Deserialize, Serialize};
use shared::utils::get_random_hash;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EnvConfig {
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PortConfig {
    pub port: u16,
    pub target_port: u16,
    pub protocol: Option<String>,
    pub r#type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Container {
    pub id: Option<String>,
    pub name: String,
    pub image: String,
    pub env: Option<Vec<EnvConfig>>,
    pub ports: Option<PortConfig>,
}

impl Container {
    pub fn get_uuid(&self) -> String {
        get_random_hash(5)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Spec {
    pub containers: Vec<Container>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkloadDefinition {
    pub api_version: String,
    pub kind: String,
    pub name: String,
    pub spec: Spec,
}

impl WorkloadDefinition {
    pub fn get_containers(&self, instance_id: &String) -> Vec<Container> {
        let mut containers = Vec::<Container>::new();
        for mut container in self.spec.containers.clone() {
            container.id = Some(format!("{}-{}-{}", instance_id, container.name, container.get_uuid()));
            containers.push(container);
        }
        containers
    }
}