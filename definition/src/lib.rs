pub mod workload {
    use serde::{Deserialize, Serialize};

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
        pub name: String,
        pub image: String,
        pub env: Option<Vec<EnvConfig>>,
        pub ports: Option<PortConfig>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Spec {
        pub containers: Vec<Container>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct WorkloadDefinition {
        pub api_version: String,
        pub kind: String,
        pub name: String,
        pub spec: Spec,
        pub replicas: Option<u16>,
    }
}
