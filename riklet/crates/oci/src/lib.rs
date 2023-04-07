use async_trait::async_trait;

pub mod image;
pub mod image_manager;
pub mod skopeo;
pub mod umoci;
use thiserror::Error;

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unable to locate the umoci binary")]
    UmociNotFoundError,
    #[error("Unable to locate the skopeo binary")]
    SkopeoNotFoundError,
    #[error("An error occured during the following spawn process: {0}")]
    ProcessSpawnError(std::io::Error),
    #[error("Umoci command timeout {0}")]
    UmociCommandTimeoutError(tokio::time::error::Elapsed),
    #[error("Skopeo command timeout {0}")]
    SkopeoCommandTimeoutError(tokio::time::error::Elapsed),
    #[error("Umoci command failed, stdout: \"{0}\", stderr: \"{1}\"")]
    UmociCommandFailedError(String, String),
    #[error("Skopeo command failed, stdout: \"{0}\", stderr: \"{1}\"")]
    SkopeoCommandFailedError(String, String),
    #[error("Umoci command error: {0}")]
    UmociCommandError(std::io::Error),
    #[error("Skopeo command error: {0}")]
    SkopeoCommandError(std::io::Error),
    #[error("Invalid path: {0}")]
    InvalidPathError(std::io::Error),
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
