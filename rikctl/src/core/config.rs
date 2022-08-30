use anyhow::{Context, Error, Result};
use config::{Config, Environment, File};
use serde::{Deserialize, Serialize};

/// `Configuration` hold the configuration of the tool
/// in order to be able to interact with the remote cluster.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Configuration {
    /// Cluster related configuration
    pub cluster: Cluster,
}

/// `Cluster` hold the configuration block at the key `cluster` in `Configuration`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cluster {
    /// The name of the cluster
    pub name: String,
    /// The endpoint of the cluster
    pub server: String,
}

impl Configuration {
    pub fn load() -> Result<Self> {
        let default_config_path = match dirs::home_dir().and_then(|p| p.to_str().map(String::from))
        {
            Some(path) => Ok(path),
            None => Err(Error::msg(
                "Wrong operating system, cannont find home directory",
            )),
        };

        let config_file = match std::env::var("RIKCONFIG") {
            Ok(value) => value,
            Err(_) => format!("{}/.rik/config.json", default_config_path?),
        };

        let config = Config::builder()
            .add_source(File::with_name(&config_file))
            .add_source(Environment::with_prefix("RIK").separator("_"))
            .build();

        config
            .map_err(Error::msg)
            .and_then(|c| {
                c.try_deserialize::<Configuration>()
                    .context("An error occurred when trying to deserialize the configuration")
            })
            .context("An error occurred when trying to load the configuration")
    }
}

impl Default for Cluster {
    fn default() -> Self {
        Self {
            name: "RIK-local".to_string(),
            server: "http://127.0.0.1:5000".to_string(),
        }
    }
}
