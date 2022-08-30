use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::core::config;
use crate::core::workload::Workload;

use super::instance::Instance;

/// `ResponseEntity` holds data about an entity
/// returned by the API.
#[derive(Debug, Deserialize, Serialize)]
pub struct ResponseEntity<T> {
    pub id: String,
    pub name: String,
    pub value: T,
}

#[async_trait]
pub trait WorkloadClient {
    async fn get_workloads(&self) -> Result<Vec<ResponseEntity<Workload>>>;
    async fn create_workload(&self, workload: &Workload) -> Result<String>;
    async fn delete_workload(&self, workload: &str) -> Result<String>;
}

#[async_trait]
pub trait InstanceClient {
    async fn get_instances(&self) -> Result<Vec<ResponseEntity<Instance>>>;
    async fn create_instance(&self, workload_id: &str, replicas: &Option<usize>) -> Result<()>;
    async fn delete_instance(&self, workload_id: &str) -> Result<String>;
}

/// `Client` provides the ability to interact
/// with the cluster controller by using HTTP Protocol.
#[derive(Debug)]
pub struct Client {
    /// The full address for accessing the cluster controller.
    ///
    /// e.g: http://127.0.0.1:5000
    endpoint: String,

    /// The internal HTTP client used to make requests.
    http_client: HttpClient,
}

impl Client {
    pub fn init(config: config::Cluster) -> Self {
        Self {
            endpoint: config.server,
            http_client: HttpClient::new(),
        }
    }

    /// Build a complete endpoint path
    pub fn endpoint(&self, path: &str) -> String {
        format!("{}/{}", self.endpoint, path)
    }
}

#[async_trait]
impl WorkloadClient for Client {
    async fn get_workloads(&self) -> Result<Vec<ResponseEntity<Workload>>> {
        let endpoint = self.endpoint("api/v0/workloads.list");
        let response = self.http_client.get(endpoint).send().await?;
        let data: Vec<ResponseEntity<Workload>> = serde_json::from_str(&response.text().await?)?;
        Ok(data)
    }

    async fn create_workload(&self, workload: &Workload) -> Result<String> {
        let endpoint = self.endpoint("api/v0/workloads.create");

        let response = self
            .http_client
            .post(endpoint)
            .body(serde_json::to_string(workload)?)
            .send()
            .await?;

        let json: Value = serde_json::from_str(&response.text().await?)?;
        Ok(json["id"].to_string())
    }

    async fn delete_workload(&self, _workload_name: &str) -> Result<String> {
        Ok(String::from("Not implemented yet"))
    }
}
#[async_trait]
impl InstanceClient for Client {
    async fn get_instances(&self) -> Result<Vec<ResponseEntity<Instance>>> {
        let endpoint = self.endpoint("api/v0/instances.list");
        let response = self.http_client.get(endpoint).send().await?;
        let data: Vec<ResponseEntity<Instance>> = serde_json::from_str(&response.text().await?)?;
        Ok(data)
    }

    async fn create_instance(&self, workload_id: &str, replicas: &Option<usize>) -> Result<()> {
        let endpoint = self.endpoint("api/v0/instances.create");

        let body = match replicas {
            Some(replicas) => json!({
                "workload_id": workload_id,
                "replicas": replicas,
            }),
            None => json!({
            "workload_id": workload_id,
            }),
        };

        let _response = self
            .http_client
            .post(endpoint)
            .body(body.to_string())
            .send()
            .await?;
        Ok(())
    }

    async fn delete_instance(&self, workload_id: &str) -> Result<String> {
        let endpoint = self.endpoint("api/v0/instances.delete");

        let body = json!({
            "id": workload_id,
        });

        let response = self
            .http_client
            .post(endpoint)
            .body(body.to_string())
            .send()
            .await?;

        let json: Value = serde_json::from_str(&response.text().await?)?;
        Ok(json.to_string())
    }
}
