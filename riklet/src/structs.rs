use serde::{Deserialize, Serialize};
use shared::utils::get_random_hash;
use tracing::{event, Level};

const DEFAULT_FUNCTION_RUNTIME_PORT: u16 = 3000;

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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct FunctionExecution {
    /// Remote URL to a RootFS, must be accessible from the runtime
    pub rootfs: url::Url,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum NetworkPortExposureType {
    /// Port will be exposed on the node fun
    NodePort,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct FunctionPort {
    /// Port used to call the function
    pub port: u16,
    /// Port exposed by the function internally
    #[serde(rename = "targetPort")]
    pub target_port: u16,
    #[serde(rename = "type")]
    pub port_type: NetworkPortExposureType,
}

impl FunctionPort {
    /// Create a FunctionPort and bind it to the default port 3000
    /// All our runtimes only use this port
    pub fn new(port: u16) -> Self {
        Self {
            port,
            target_port: DEFAULT_FUNCTION_RUNTIME_PORT,
            port_type: NetworkPortExposureType::NodePort,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub execution: FunctionExecution,
    pub exposure: Option<FunctionPort>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Spec {
    pub containers: Vec<Container>,
    pub function: Option<Function>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkloadDefinition {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,
    pub name: String,
    pub spec: Spec,
}

impl WorkloadDefinition {
    pub fn get_containers(&self, instance_id: &str) -> Vec<Container> {
        let mut containers = Vec::<Container>::new();
        for mut container in self.spec.containers.clone() {
            event!(Level::DEBUG, "Container: {:?}", &container);
            container.id = Some(format!(
                "{}-{}-{}",
                instance_id,
                container.name,
                container.get_uuid()
            ));
            containers.push(container);
        }
        containers
    }
}
