use async_trait::async_trait;
use log::error;
use snafu::Snafu;

pub mod image;
pub mod image_manager;
pub mod skopeo;
pub mod umoci;

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(display("Unable to locate the umoci binary"))]
    UmociNotFoundError {},
    #[snafu(display("Unable to locate the skopeo binary"))]
    SkopeoNotFoundError {},
    #[snafu(display("An error occured during the following spawn process: {}", source))]
    ProcessSpawnError { source: std::io::Error },
    #[snafu(display("Umoci command timeout {}", source))]
    UmociCommandTimeoutError { source: tokio::time::error::Elapsed },
    #[snafu(display("Skopeo command timeout {}", source))]
    SkopeoCommandTimeoutError { source: tokio::time::error::Elapsed },
    #[snafu(display("Umoci command failed, stdout: \"{}\", stderr: \"{}\"", stdout, stderr))]
    UmociCommandFailedError { stdout: String, stderr: String },
    #[snafu(display(
        "Skopeo command failed, stdout: \"{}\", stderr: \"{}\"",
        stdout,
        stderr
    ))]
    SkopeoCommandFailedError { stdout: String, stderr: String },
    #[snafu(display("Umoci command error: {}", source))]
    UmociCommandError { source: std::io::Error },
    #[snafu(display("Skopeo command error: {}", source))]
    SkopeoCommandError { source: std::io::Error },
    #[snafu(display("Invalid path: {}", source))]
    InvalidPathError { source: std::io::Error },
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
