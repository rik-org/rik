pub mod workload {
    use serde::{Deserialize, Serialize};
    use std::fmt::Display;

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
                target_port: 3000,
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
}
