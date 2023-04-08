use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{event, Level};

pub mod console;
pub mod container;
use thiserror::Error;

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unable to locate the runc binary")]
    RuncNotFoundError,
    #[error("An error occured during the following spawn process: {0}")]
    ProcessSpawnError(std::io::Error),
    #[error("Runc command timeout {0}")]
    RuncCommandTimeoutError(tokio::time::error::Elapsed),
    #[error("Runc command failed, stdout: \"{0}\", stderr: \"{1}\"")]
    RuncCommandFailedError(String, String),
    #[error("Runc command error: {0}")]
    RuncCommandError(std::io::Error),
    #[error("Invalid path: {0}")]
    InvalidPathError(std::io::Error),
    #[error("Unable to bind to unix socket: {0}")]
    UnixSocketOpenError(std::io::Error),
    #[error("Json deserialization error: {0}")]
    JsonDeserializationError(serde_json::error::Error),
}

trait Args {
    fn args(&self) -> Result<Vec<String>>;
}

/// A trait to implement executable
#[async_trait]
trait Executable: Args {
    async fn exec(&self, args: &[String]) -> Result<String>;

    fn concat_args(&self, args: &[String]) -> Result<Vec<String>> {
        let mut combined = self.args()?;
        combined.append(&mut args.iter().cloned().map(String::from).collect());
        Ok(combined)
    }

    fn append_opts(args: &mut Vec<String>, opts: Option<&dyn Args>) -> Result<()>
    where
        Self: Sized,
    {
        if let Some(opts) = opts {
            args.append(&mut opts.args()?);
        }
        Ok(())
    }
}

/// Runc container
#[derive(Debug, Serialize, Deserialize)]
pub struct Container {
    /// Container id
    pub id: Option<String>,
    /// Process id
    pub pid: Option<usize>,
    /// Current status
    pub status: Option<String>,
    /// OCI bundle path
    pub bundle: Option<String>,
    /// Root filesystem path
    pub rootfs: Option<String>,
    /// Creation time
    pub created: Option<DateTime<Utc>>,
    /// Annotations
    pub annotations: Option<HashMap<String, String>>,
}
