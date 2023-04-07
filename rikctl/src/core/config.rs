use std::path::PathBuf;

use anyhow::{Error, Result};
use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};

const CONFIG_LOCATION_KEY: &str = "RIKCONFIG";
const CONFIG_FILE_NAME: &str = "config.yaml";

/// `Configuration` hold the configuration of the tool
/// in order to be able to interact with the remote cluster.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Configuration {
    pub cluster: Cluster,
}

/// `Cluster` hold the configuration block at the key `cluster` in `Configuration`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cluster {
    pub name: String,
    pub server: String,
}

impl Default for Cluster {
    fn default() -> Self {
        Self {
            name: "rik.local".to_string(),
            server: "http://127.0.0.1:5000".to_string(),
        }
    }
}

impl Configuration {
    #[cfg(not(target_os = "windows"))]
    fn get_home_path() -> Option<String> {
        dirs::home_dir().and_then(|p| p.to_str().map(String::from))
    }

    // We do not support Windows configuration for now
    #[cfg(target_os = "windows")]
    fn get_home_path() -> Option<String> {
        None
    }

    fn default_config_path() -> PathBuf {
        match Self::get_home_path() {
            Some(path) => PathBuf::from(path),
            None => PathBuf::from("."),
        }
    }

    /// Write a default configuration file on the system
    fn create_default_config() -> Result<()> {
        let config = Configuration::default();
        let config_file = Self::get_home_path()
            .ok_or(Error::msg("Could not find home directory"))
            .map(|p| PathBuf::from(p))?;

        let full_config_path = config_file.join(".rik").join(CONFIG_FILE_NAME);

        std::fs::create_dir_all(full_config_path.parent().unwrap())?;
        std::fs::write(full_config_path, serde_yaml::to_string(&config)?)?;
        Ok(())
    }

    fn deserialize_or_default(config: Config, is_default_path: bool) -> Result<Configuration> {
        match config.try_deserialize::<Configuration>() {
            Ok(config) => Ok(config),
            // If the configuration is invalid, we throw an error about it
            Err(ConfigError::FileParse { uri, cause }) => Err(Error::msg(format!(
                "Could not parse configuration file: {}, reason: {}",
                uri.unwrap(),
                cause
            ))),
            // If any other error occurs (mostly because configuration doesn't exist), we create a default configuration file
            Err(_) if is_default_path => Self::create_default_config().and_then(|_| Self::load()),
            Err(e) => Err(Error::new(e)),
        }
    }

    pub fn load() -> Result<Self> {
        let config_file = Self::default_config_path()
            .join(".rik")
            .join(CONFIG_FILE_NAME)
            .to_string_lossy()
            .to_string();

        // Config won't throw any error in case no config is found (weird?)
        let mut config = Config::builder()
            // Configuration default location
            .add_source(File::new(config_file.as_str(), config::FileFormat::Yaml).required(false))
            .add_source(Environment::with_prefix("RIK").separator("_"));

        // Configuration file location provided by the user through env var
        match std::env::var(CONFIG_LOCATION_KEY) {
            Ok(key) => {
                config = config.add_source(File::new(key.as_str(), config::FileFormat::Yaml));
            }
            Err(_) => (),
        };

        match config.build() {
            Err(e) => Err(Error::new(e)),
            Ok(c) => Self::deserialize_or_default(c, std::env::var(CONFIG_LOCATION_KEY).is_err()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_config_from_string(config: &str) -> NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(config.as_bytes()).unwrap();
        file
    }

    #[test]
    #[serial]
    fn provide_config_through_env() {
        std::env::set_var("RIK_CLUSTER_NAME", "test");
        std::env::set_var("RIK_CLUSTER_SERVER", "http://test.com");

        let config = Configuration::load().unwrap();

        assert_eq!(config.cluster.name, "test");
        assert_eq!(config.cluster.server, "http://test.com");
        std::env::remove_var("RIK_CLUSTER_NAME");
        std::env::remove_var("RIK_CLUSTER_SERVER");
    }

    #[test]
    #[serial]
    fn provide_config_default() {
        let config = Configuration::load().unwrap();
        assert_eq!(config.cluster.name, "rik.local");
        assert_eq!(config.cluster.server, "http://127.0.0.1:5000");
    }

    #[test]
    #[serial]
    fn provide_config_location_env() {
        let config_str = r#"
cluster:
    name: test
    server: http://test.com
        "#;
        let _config_file = write_config_from_string(config_str);
        let path = _config_file.path().to_string_lossy().to_string();

        std::env::set_var(CONFIG_LOCATION_KEY, path.clone());
        let config = Configuration::load().expect("Should be able to load configuration");
        assert_eq!(config.cluster.name, "test");
        assert_eq!(config.cluster.server, "http://test.com");
        std::env::remove_var(CONFIG_LOCATION_KEY);
    }
}
