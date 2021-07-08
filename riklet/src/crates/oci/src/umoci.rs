use std::path::PathBuf;
use shared::utils::find_binary;
use snafu::{OptionExt, ResultExt, ensure};
use std::time::Duration;
use log::debug;
use crate::*;
use tokio::process::Command;
use std::process::Stdio;
use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UmociConfiguration {
    pub debug: bool,
    pub command: Option<PathBuf>,
    pub bundles_directory: Option<PathBuf>,
    pub timeout: Option<Duration>,
    pub log_level: Option<String>,
}

/// Implementation of umoci
#[derive(Debug)]
pub struct Umoci {
    command: PathBuf,
    timeout: Duration,
    bundles_directory: PathBuf,
    verbose: bool,
    log_level: Option<String>
}

impl Umoci {

    /// Create an Umoci instance to interact with the binary
    pub fn new(config: UmociConfiguration) -> Result<Self> {
        let command = config
            .command
            .or_else(|| find_binary("umoci"))
            .context(UmociNotFoundError {})?;

        let timeout = config.timeout.or(Some(Duration::from_millis(5000))).unwrap();

        let bundles_directory = config
            .bundles_directory
            .unwrap()
            .canonicalize()
            .context(InvalidPathError {})?;

        debug!("Umoci initialized.");

        Ok(Self {
            command,
            timeout,
            bundles_directory,
            log_level: config.log_level,
            verbose: config.debug,
        })
    }

    fn get_bundle_path(&self, bundle_id: &String) -> String {
        format!(
            "{}/{}",
            self.bundles_directory.to_str().unwrap(),
            bundle_id
        )
    }

    pub async fn unpack(&self, bundle_id: &String, opts: Option<&UnpackArgs>) -> Result<String> {
        let mut args = vec![String::from("unpack")];
        Self::append_opts(&mut args, opts.map(|opts| opts as &dyn Args))?;
        let bundle_path = self.get_bundle_path(bundle_id);

        args.push(bundle_path.clone());

        self.exec(&args).await?;

        Ok(bundle_path)
    }
}

impl Args for Umoci {
    fn args(&self) -> Result<Vec<String>> {
        let mut args: Vec<String> = Vec::new();

        if self.verbose {
            args.push(String::from("--verbose"));
        }

        if let Some(log_level) = &self.log_level {
            args.push(String::from("--log"));
            args.push(String::from(log_level.to_string()));
        }

        Ok(args)
    }
}

#[async_trait]
impl Executable for Umoci {
    async fn exec(&self, args: &[String]) -> Result<String> {
        let args = self.concat_args(args)?;
        let process = Command::new(&self.command)
            .args(&args.clone())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context(ProcessSpawnError {})?;

        debug!("{} {}", self.command.to_str().unwrap(), &args.clone().join(" "));

        let result = tokio::time::timeout(self.timeout, process.wait_with_output())
            .await
            .context(UmociCommandTimeoutError {})?
            .context(UmociCommandError {})?;

        let stdout = String::from_utf8(result.stdout.clone()).unwrap();
        let stderr = String::from_utf8(result.stderr.clone()).unwrap();

        if stderr != "" {
            if stderr.contains("config.json already exists") {
                log::warn!("A config.json already exists for this image.");
            } else {
                error!("Umoci error : {}", stderr);
                ensure!(
                    result.status.success(),
                    UmociCommandFailedError {
                        stdout: stdout,
                        stderr: stderr
                    }
                );
            }
        }

        Ok(stdout)
    }
}

pub struct UnpackArgs {
    pub keep_dirlinks: bool,
    pub uid_map: Option<String>,
    pub gid_map: Option<String>,
    pub rootless: bool,
    pub image: PathBuf
}

impl Args for UnpackArgs {
    fn args(&self) -> Result<Vec<String>> {
        let mut args: Vec<String> = Vec::new();

        if self.keep_dirlinks {
            args.push(String::from("--keep-dirlinks"));
        }

        if let Some(uid_map) = &self.uid_map {
            args.push(String::from("--uid-map"));
            args.push(uid_map.to_string());
        }

        if let Some(gid_map) = &self.gid_map {
            args.push(String::from("--gid-map"));
            args.push(gid_map.to_string());
        }

        if self.rootless {
            args.push(String::from("--rootless"));
        }

        args.push(String::from("--image"));
        args.push(String::from(self.image.to_str().unwrap()));

        Ok(args)
    }
}