use crate::*;
use serde::{Deserialize, Serialize};
use shared::utils::find_binary;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;

#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct RuncConfiguration {
    /// Ignore cgroup permission errors
    pub rootless: bool,
    /// Enable debug output for logging
    pub debug: bool,
    /// Timeout for Runc commands.
    pub timeout: Option<Duration>,
    /// The path of the runc command. If None, we will try to search the package in your $PATH.
    pub command: Option<PathBuf>,
    /// The root directory for storage of container state
    pub root: Option<PathBuf>,
}

/// A basic implementation to interact with the Runc binary
#[derive(Debug)]
pub struct Runc {
    command: PathBuf,
    timeout: Duration,
    root: Option<PathBuf>,
    rootless: bool,
    debug: bool,
}

impl Runc {
    /// Create a Runc instance with the provided configuration.
    pub fn new(config: RuncConfiguration) -> Result<Self> {
        event!(Level::DEBUG, "Initializing Runc...");
        let command = config
            .command
            .or_else(|| find_binary("runc"))
            .ok_or(Error::RuncNotFoundError)?;

        let timeout = config
            .timeout
            .or_else(|| Some(Duration::from_millis(5000)))
            .unwrap();

        event!(Level::DEBUG, "Runc initialized.");

        Ok(Self {
            command,
            timeout,
            root: config.root,
            debug: config.debug,
            rootless: config.rootless,
        })
    }

    /// List all containers
    pub async fn list(&self) -> Result<Vec<Container>> {
        let args = vec![String::from("list"), String::from("--format=json")];
        let mut output = self.exec(&args).await?;
        output = output.trim().to_string();

        Ok(if output == "null" {
            Vec::new()
        } else {
            serde_json::from_str(&output).unwrap()
        })
    }

    /// Send the specified signal to all processes inside the container.
    pub async fn kill(&self, id: &str, sig: i32, opts: Option<&KillArgs>) -> Result<()> {
        event!(Level::DEBUG, "Killing container {}", id);
        let mut args = vec![String::from("kill")];
        Self::append_opts(&mut args, opts.map(|opts| opts as &dyn Args))?;
        args.push(String::from(id));
        args.push(format!("{}", sig));
        self.exec(&args).await.map(|_| ())
    }

    /// Run a container.
    pub async fn run(&self, id: &str, bundle: &Path, opts: Option<&CreateArgs>) -> Result<()> {
        event!(Level::DEBUG, "Running container {}", id);
        let mut args = vec![String::from("run")];
        Self::append_opts(&mut args, opts.map(|opts| opts as &dyn Args))?;

        let bundle: String = bundle
            .canonicalize()
            .map_err(Error::InvalidPathError)?
            .to_string_lossy()
            .parse()
            .unwrap();

        args.push(String::from("--bundle"));
        args.push(bundle);
        args.push(String::from(id));
        self.exec(&args).await.map(|_| ())
    }

    /// Get the state of a container
    pub async fn state(&self, id: &str) -> Result<Container> {
        let args = vec![String::from("state"), String::from(id)];
        let output = self.exec(&args).await?;
        serde_json::from_str(&output).map_err(Error::JsonDeserializationError)
    }

    /// Delete a container
    pub async fn delete(&self, id: &str, opts: Option<&DeleteArgs>) -> Result<()> {
        event!(Level::DEBUG, "Deleting container {}", id);
        let mut args = vec![String::from("delete")];
        Self::append_opts(&mut args, opts.map(|opts| opts as &dyn Args))?;
        args.push(String::from(id));
        self.exec(&args).await.map(|_| ())
    }
}

impl Args for Runc {
    /// Implement arguments for Runc binary.
    fn args(&self) -> Result<Vec<String>> {
        let mut args: Vec<String> = Vec::new();

        if let Some(root) = self.root.clone() {
            args.push(String::from("--root"));
            args.push(
                root.canonicalize()
                    .map_err(Error::InvalidPathError)?
                    .to_string_lossy()
                    .parse()
                    .unwrap(),
            )
        }

        if self.rootless {
            args.push(format!("--rootless={}", self.rootless))
        }

        if self.debug {
            args.push(String::from("--debug"));
        }

        Ok(args)
    }
}

#[async_trait]
impl Executable for Runc {
    async fn exec(&self, args: &[String]) -> Result<String> {
        let args = self.concat_args(args)?;
        let process = Command::new(&self.command)
            .args(&args.clone())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(Error::ProcessSpawnError)?;

        event!(
            Level::DEBUG,
            "{} {}",
            self.command.to_str().unwrap(),
            &args.clone().join(" ")
        );

        let result = tokio::time::timeout(self.timeout, process.wait_with_output())
            .await
            .map_err(Error::RuncCommandTimeoutError)?
            .map_err(Error::RuncCommandError)?;

        let stdout = String::from_utf8(result.stdout.clone()).unwrap();
        let stderr = String::from_utf8(result.stderr.clone()).unwrap();

        if !stderr.is_empty() {
            event!(Level::ERROR, "Runc error : {}", stderr);
        }

        if !result.status.success() {
            return Err(Error::RuncCommandFailedError(stdout, stderr));
        }

        Ok(stdout)
    }
}

/// runc create arguments
#[derive(Debug, Clone)]
pub struct CreateArgs {
    pub pid_file: Option<PathBuf>,
    pub console_socket: Option<PathBuf>,
    pub no_pivot: bool,
    pub no_new_keyring: bool,
    pub detach: bool,
}

impl Args for CreateArgs {
    fn args(&self) -> Result<Vec<String>> {
        let mut args: Vec<String> = Vec::new();

        if let Some(pid_file) = self.pid_file.clone() {
            args.push(String::from("--pid-file"));
            args.push(pid_file.to_string_lossy().parse().unwrap())
        }

        if let Some(console_socket) = self.console_socket.clone() {
            args.push(String::from("--console-socket"));
            args.push(
                console_socket
                    .canonicalize()
                    .map_err(Error::InvalidPathError)?
                    .to_string_lossy()
                    .parse()
                    .unwrap(),
            )
        }

        if self.no_pivot {
            args.push(String::from("--no-pivot"))
        }

        if self.no_new_keyring {
            args.push(String::from("--no-new-keyring"))
        }

        if self.detach {
            args.push(String::from("--detach"))
        }

        Ok(args)
    }
}

#[derive(Debug, Clone)]
pub struct KillArgs {
    /// Send the specified signal to all processes inside the container
    pub all: bool,
}

impl Args for KillArgs {
    fn args(&self) -> Result<Vec<String>> {
        let mut args: Vec<String> = Vec::new();
        if self.all {
            args.push(String::from("--all"))
        }

        Ok(args)
    }
}

/// runc delete arguments
pub struct DeleteArgs {
    pub force: bool,
}

impl Args for DeleteArgs {
    fn args(&self) -> Result<Vec<String>> {
        let mut args: Vec<String> = Vec::new();

        if self.force {
            args.push(String::from("--force"));
        }

        Ok(args)
    }
}

// Clean up after tests
#[cfg(test)]
impl Drop for Runc {
    fn drop(&mut self) {
        if let Some(root) = self.root.clone() {
            if let Err(e) = std::fs::remove_dir_all(&root) {
                event!(Level::ERROR, "failed to cleanup root directory: {}", e);
            }
        }
        if let Some(system_runc) = find_binary("runc") {
            if system_runc != self.command {
                if let Err(e) = std::fs::remove_file(&self.command) {
                    event!(Level::ERROR, "failed to remove runc binary: {}", e);
                }
            }
        } else if let Err(e) = std::fs::remove_file(&self.command) {
            event!(Level::ERROR, "failed to remove runc binary: {}", e);
        }
    }
}

/// these tests should be ignored because we can't run Runc container in CI pipeline.
/// To run, use the following `cargo test --workspace --ignored`
#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use std::env::temp_dir;
    use std::fs::{copy, create_dir_all};
    use std::path::PathBuf;

    use crate::console::ConsoleSocket;
    use crate::container::{CreateArgs, DeleteArgs, Runc, RuncConfiguration};
    use shared::utils::unpack;
    use std::time::Duration;
    use tokio::time::sleep;
    use tracing::{event, Level};

    const BUSYBOX_ARCHIVE: &str = "../../../fixtures/busybox.tar.gz";
    const RUNC_FIXTURE: &str = "../../../fixtures/runc.amd64";

    struct TestContainer {
        id: String,
        runc: Option<Runc>,
    }

    impl TestContainer {
        async fn new(
            runc_path: &PathBuf,
            runc_root: &PathBuf,
            archive_bundle: &PathBuf,
        ) -> crate::Result<Self> {
            let id = format!("{}", Uuid::new_v4());
            let bundle = temp_dir().join(&id);

            unpack(archive_bundle.to_str().unwrap(), &bundle).expect("Unable to extract bundle");

            let mut config: RuncConfiguration = Default::default();
            config.command = Some(runc_path.clone());
            config.root = Some(runc_root.clone());

            let runc = Runc::new(config)?;

            let socket_path = temp_dir().join(&id).with_extension("console");
            let console_socket =
                ConsoleSocket::new(&socket_path).expect("Unable to create the console socket.");

            tokio::spawn(async move {
                match console_socket
                    .get_listener()
                    .as_ref()
                    .unwrap()
                    .accept()
                    .await
                {
                    Ok((stream, _socket_addr)) => {
                        Box::leak(Box::new(stream));
                    }
                    Err(err) => {
                        event!(Level::ERROR, "Receive PTY master error : {:?}", err)
                    }
                }
            });

            runc.run(
                &id,
                &bundle,
                Some(&CreateArgs {
                    detach: true,
                    console_socket: Some(socket_path),
                    no_new_keyring: false,
                    no_pivot: false,
                    pid_file: None,
                }),
            )
            .await?;

            Ok(Self {
                runc: Some(runc),
                id,
            })
        }
    }

    /// Install Runc in the temporary environment for the test & create all directories and files used by the test.
    fn setup_test_sequence() -> (PathBuf, PathBuf) {
        let sequence_id = format!("{}", Uuid::new_v4());
        let mut sequence_path = temp_dir().join(&sequence_id);
        let sequence_root = temp_dir().join("runc").join(&sequence_id);

        create_dir_all(&sequence_root).expect("Unable to create runc root");
        create_dir_all(&sequence_path).expect("Unable to create the temporary folder");

        sequence_path = sequence_path.join("runc.amd64");

        copy(PathBuf::from(RUNC_FIXTURE), &sequence_path)
            .expect("Unable to copy runc binary into the temporary folder.");

        (sequence_path, sequence_root)
    }

    #[tokio::test]
    #[ignore]
    async fn test_it_run_a_container() {
        let (runc_path, runc_root) = setup_test_sequence();

        let mut config: RuncConfiguration = Default::default();
        config.command = Some(runc_path);
        config.root = Some(runc_root);

        let runc = Runc::new(config).expect("Unable to create runc instance");

        let id = format!("{}", Uuid::new_v4());
        let socket_path = temp_dir().join(&id).with_extension("console");
        let console_socket =
            ConsoleSocket::new(&socket_path).expect("Unable to create the console socket.");

        tokio::spawn(async move {
            match console_socket
                .get_listener()
                .as_ref()
                .unwrap()
                .accept()
                .await
            {
                Ok((stream, _socket_addr)) => {
                    Box::leak(Box::new(stream));
                }
                Err(err) => {
                    event!(Level::ERROR, "Receive PTY master error : {:?}", err)
                }
            }
        });

        let bundle = temp_dir().join(&id);

        let _ = unpack(BUSYBOX_ARCHIVE, &bundle);

        runc.run(
            &id,
            &bundle,
            Some(&CreateArgs {
                pid_file: None,
                console_socket: Some(socket_path),
                no_pivot: false,
                no_new_keyring: false,
                detach: true,
            }),
        )
        .await
        .expect("Failed to run the container");

        sleep(Duration::from_millis(500)).await;

        let container = runc
            .state(&id)
            .await
            .expect("Unable to get the state of the container");

        assert_eq!(container.status, Some(String::from("running")))
    }

    #[tokio::test]
    #[ignore]
    async fn test_it_delete_a_container() {
        let (runc_path, runc_root) = setup_test_sequence();

        let container = TestContainer::new(&runc_path, &runc_root, &PathBuf::from(BUSYBOX_ARCHIVE))
            .await
            .expect("Unable to create the container");

        let runc = container.runc.unwrap();

        runc.kill(&container.id, libc::SIGKILL, None)
            .await
            .expect("Unable to kill the container");

        sleep(Duration::from_millis(500)).await;

        runc.delete(&container.id, None)
            .await
            .expect("Unable to delete the container");

        let containers = runc.list().await.expect("Unable to list containers");

        assert!(containers.is_empty())
    }

    #[tokio::test]
    #[ignore]
    async fn test_it_force_delete_a_container() {
        let (runc_path, runc_root) = setup_test_sequence();

        let container = TestContainer::new(&runc_path, &runc_root, &PathBuf::from(BUSYBOX_ARCHIVE))
            .await
            .expect("Unable to create the container");

        let runc = container.runc.unwrap();

        sleep(Duration::from_millis(500)).await;

        runc.delete(&container.id, Some(&DeleteArgs { force: true }))
            .await
            .expect("Unable to delete the container");

        let containers = runc.list().await.expect("Unable to list containers");

        assert!(containers.is_empty())
    }

    #[tokio::test]
    #[ignore]
    async fn test_it_kill_a_container() {
        let (runc_path, runc_root) = setup_test_sequence();

        let container = TestContainer::new(&runc_path, &runc_root, &PathBuf::from(BUSYBOX_ARCHIVE))
            .await
            .expect("Unable to create the container");

        let runc = container.runc.unwrap();

        runc.kill(&container.id, libc::SIGKILL, None)
            .await
            .expect("Unable to kill the container");

        sleep(Duration::from_millis(500)).await;

        let container_state = runc
            .state(&container.id)
            .await
            .expect("Unable to get the container state");

        assert_eq!(container_state.status, Some(String::from("stopped")))
    }
}
