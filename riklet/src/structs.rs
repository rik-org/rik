use serde::{Deserialize, Serialize};
use shared::utils::get_random_hash;
use tracing::{event, warn, Level};

#[async_trait::async_trait]
pub trait EventEmitter<U, T> {
    async fn emit_event(
        mut client: T,
        event: U,
    ) -> std::result::Result<(), Box<dyn std::error::Error>>;
}

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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub execution: FunctionExecution,
    pub exposure: Option<FunctionPort>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Spec {
    pub containers: Vec<Container>,
    pub function: Option<Function>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

    pub fn get_rootfs_url(&self) -> Option<String> {
        self.spec
            .function
            .as_ref()
            .map(|v| v.execution.rootfs.to_string())
    }

    /// Give expected ports exposed by the workload.
    /// Returns a tuple of (host_port, target_port)
    #[tracing::instrument(skip(self), fields(self.name))]
    pub fn get_port_mapping(&self) -> Vec<(u16, u16)> {
        let mut port_mapping = Vec::<(u16, u16)>::new();
        let function_exposure = self
            .spec
            .function
            .as_ref()
            .and_then(|f| f.exposure.as_ref().map(|e| (e.port, e.target_port)));

        if let Some((host_port, target_port)) = function_exposure {
            // FIXME: This is a domain violation, as we want to get away from binding to this "FUNCTION_RUNTIME" things
            // which refers to a specific implementation of a VM
            port_mapping.push((host_port, target_port));
        } else {
            warn!("No port mapping found for workload {}", self.name);
        }
        port_mapping
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workload_function_port_mapping() {
        let workload = WorkloadDefinition {
            api_version: "v1".to_string(),
            kind: "WorkloadDefinition".to_string(),
            name: "test".to_string(),
            spec: Spec {
                containers: vec![],
                function: Some(Function {
                    execution: FunctionExecution {
                        rootfs: url::Url::parse("http://localhost:8080").unwrap(),
                    },
                    exposure: Some(FunctionPort {
                        port: 8080,
                        target_port: 8081,
                        port_type: NetworkPortExposureType::NodePort,
                    }),
                }),
            },
        };

        let port_mapping = workload.get_port_mapping();
        assert_eq!(port_mapping.len(), 1);
        // host port
        assert_eq!(port_mapping[0].0, 8080);
        // internal port
        assert_eq!(port_mapping[0].1, 8081);
    }

    #[test]
    fn test_workload_no_function_port_mapping() {
        let workload = WorkloadDefinition {
            api_version: "v1".to_string(),
            kind: "WorkloadDefinition".to_string(),
            name: "test".to_string(),
            spec: Spec {
                containers: vec![],
                function: None,
            },
        };

        let port_mapping = workload.get_port_mapping();
        assert_eq!(port_mapping.len(), 0);
    }
}
