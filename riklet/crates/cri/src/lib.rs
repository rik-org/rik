use async_trait::async_trait;
use chrono::{DateTime, Utc};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use snafu::{OptionExt, ResultExt, Snafu};
use std::collections::HashMap;
use std::path::PathBuf;

pub mod console;
pub mod container;

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(display("Unable to locate the runc binary"))]
    RuncNotFoundError {},
    #[snafu(display("An error occured during the following spawn process: {}", source))]
    ProcessSpawnError { source: std::io::Error },
    #[snafu(display("Runc command timeout {}", source))]
    RuncCommandTimeoutError { source: tokio::time::error::Elapsed },
    #[snafu(display("Runc command failed, stdout: \"{}\", stderr: \"{}\"", stdout, stderr))]
    RuncCommandFailedError { stdout: String, stderr: String },
    #[snafu(display("Runc command error: {}", source))]
    RuncCommandError { source: std::io::Error },
    #[snafu(display("Invalid path: {}", source))]
    InvalidPathError { source: std::io::Error },
    #[snafu(display("Unable to bind to unix socket: {}", source))]
    UnixSocketOpenError { source: std::io::Error },
    #[snafu(display("Json deserialization error: {}", source))]
    JsonDeserializationError { source: serde_json::error::Error },
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
