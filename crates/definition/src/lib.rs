use std::fmt::Display;

use serde::{Deserialize, Serialize};

pub mod workload {
    use serde::{Deserialize, Serialize};
    use std::fmt::Display;
    use tracing::error;

    const DEFAULT_FUNCTION_RUNTIME_PORT: u16 = 8080;

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    pub struct EnvConfig {
        pub name: String,
        pub value: String,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    pub struct PortConfig {
        pub port: u16,
        pub target_port: u16,
        pub protocol: Option<String>,
        pub r#type: String,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    pub struct Container {
        pub name: String,
        pub image: String,
        pub env: Option<Vec<EnvConfig>>,
        pub ports: Option<PortConfig>,
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

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    pub struct Spec {
        #[serde(default)]
        pub containers: Vec<Container>,
        #[serde(default)]
        pub function: Option<Function>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    pub enum WorkloadKind {
        /// A container
        Pod,
        /// A function executing a piece of code in a VM
        Function,
    }

    impl From<String> for WorkloadKind {
        fn from(kind: String) -> Self {
            match kind.as_str() {
                "Pod" => WorkloadKind::Pod,
                "Function" => WorkloadKind::Function,
                _ => panic!("Unknown workload kind"),
            }
        }
    }

    impl Display for WorkloadKind {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                WorkloadKind::Pod => write!(f, "Pod"),
                WorkloadKind::Function => write!(f, "Function"),
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
    pub struct WorkloadDefinition {
        #[serde(rename = "apiVersion")]
        pub api_version: String,
        pub kind: WorkloadKind,
        pub name: String,
        pub spec: Spec,
        pub replicas: Option<u16>,
    }

    impl WorkloadDefinition {
        /// Determine whether the workload is a kind function
        pub fn is_function(&self) -> bool {
            self.kind == WorkloadKind::Function
        }

        pub fn set_function_port(&mut self, port: u16) {
            if !self.is_function() {
                error!("Cannot set function port on non-function workload");
            }
            if let Some(function) = &mut self.spec.function {
                function.exposure = Some(FunctionPort::new(port));
            }
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

impl From<InstanceStatus> for i32 {
    fn from(value: InstanceStatus) -> Self {
        match value {
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
