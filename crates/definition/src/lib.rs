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
    pub struct Spec {
        pub containers: Vec<Container>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    pub enum WorkloadKind {
        Pod,
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
        pub api_version: String,
        pub kind: WorkloadKind,
        pub name: String,
        pub spec: Spec,
        pub replicas: Option<u16>,
    }
}
