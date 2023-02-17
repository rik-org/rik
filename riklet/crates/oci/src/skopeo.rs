use crate::*;
use serde::{Deserialize, Serialize};
use shared::utils::find_binary;
use snafu::ensure;
use snafu::{OptionExt, ResultExt};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tracing::{event, Level};

#[derive(Default, Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct SkopeoConfiguration {
    pub debug: bool,
    pub insecure_policy: bool,
    pub command: Option<PathBuf>,
    pub images_directory: Option<PathBuf>,
    pub override_arch: Option<String>,
    pub override_os: Option<String>,
    pub override_variant: Option<String>,
    pub policy: Option<String>,
    pub registries: Option<PathBuf>,
    pub tmp_dir: Option<PathBuf>,
    pub timeout: Option<Duration>,
}

#[derive(Debug)]
pub struct Skopeo {
    command: PathBuf,
    timeout: Duration,
    images_directory: PathBuf,
    debug: bool,
    insecure_policy: bool,
    override_arch: Option<String>,
    override_os: Option<String>,
    override_variant: Option<String>,
    policy: Option<String>,
    registries: Option<PathBuf>,
    tmp_dir: Option<PathBuf>,
}

impl Skopeo {
    pub fn new(config: SkopeoConfiguration) -> Result<Self> {
        event!(Level::DEBUG, "Initializing Skopeo...");
        let command = config
            .command
            .or_else(|| find_binary("skopeo"))
            .context(SkopeoNotFoundError {})?;

        let timeout = config
            .timeout
            .or_else(|| Some(Duration::from_millis(5000)))
            .unwrap();

        let images_directory = config
            .images_directory
            .unwrap()
            .canonicalize()
            .context(InvalidPathError {})?;

        event!(Level::DEBUG, "Skopeo initialized.");

        Ok(Self {
            command,
            timeout,
            images_directory,
            debug: config.debug,
            insecure_policy: config.insecure_policy,
            override_arch: config.override_arch,
            override_os: config.override_os,
            override_variant: config.override_variant,
            policy: config.policy,
            registries: config.registries,
            tmp_dir: config.tmp_dir,
        })
    }

    fn get_pull_path(&self, directory: &str) -> String {
        format!(
            "oci:{}/{}",
            self.images_directory.to_str().unwrap(),
            directory
        )
    }

    pub async fn copy(&self, src: &str, uuid: &str, opts: Option<&CopyArgs>) -> Result<String> {
        event!(Level::DEBUG, "Copying image from {} to {}", src, uuid);
        let mut args = vec![String::from("copy"), src.to_string()];
        Self::append_opts(&mut args, opts.map(|opts| opts as &dyn Args))?;

        let image_pull_path = self.get_pull_path(uuid);

        args.push(image_pull_path.clone());

        self.exec(&args).await?;

        let splitted = image_pull_path.split(':').collect::<Vec<&str>>();
        Ok(String::from(*splitted.get(1).unwrap()))
    }
}

impl Args for Skopeo {
    fn args(&self) -> Result<Vec<String>> {
        let mut args: Vec<String> = Vec::new();

        if self.debug {
            args.push(String::from("--debug"));
        }

        if self.insecure_policy {
            args.push(String::from("--insecure-policy"));
        }

        if let Some(override_arch) = self.override_arch.clone() {
            args.push(String::from("--override-arch"));
            args.push(override_arch)
        }

        if let Some(override_os) = self.override_os.clone() {
            args.push(String::from("--override-os"));
            args.push(override_os);
        }

        if let Some(override_variant) = self.override_variant.clone() {
            args.push(String::from("--override-variant"));
            args.push(override_variant);
        }

        if let Some(policy) = self.policy.clone() {
            args.push(String::from("--policy"));
            args.push(policy);
        }

        if let Some(registries) = self.registries.clone() {
            let registries = registries.canonicalize().context(InvalidPathError {})?;
            args.push(String::from("--registries.d"));
            args.push(String::from(registries.to_str().unwrap()));
        }

        if let Some(tmp_dir) = self.tmp_dir.clone() {
            let tmp_dir = tmp_dir.canonicalize().context(InvalidPathError {})?;
            args.push(String::from("--tmpdir"));
            args.push(String::from(tmp_dir.to_str().unwrap()));
        }

        Ok(args)
    }
}

#[async_trait]
impl Executable for Skopeo {
    async fn exec(&self, args: &[String]) -> Result<String> {
        let args = self.concat_args(args)?;
        let process = Command::new(&self.command)
            .args(&args.clone())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context(ProcessSpawnError {})?;

        event!(
            Level::DEBUG,
            "{} {}",
            self.command.to_str().unwrap(),
            &args.clone().join(" ")
        );

        let result = tokio::time::timeout(self.timeout, process.wait_with_output())
            .await
            .context(SkopeoCommandTimeoutError {})?
            .context(SkopeoCommandError {})?;

        let stdout = String::from_utf8(result.stdout.clone()).unwrap();
        let stderr = String::from_utf8(result.stderr.clone()).unwrap();

        if !stderr.is_empty() {
            event!(Level::ERROR, "Skopeo error : {}", stderr);
        }

        ensure!(
            result.status.success(),
            SkopeoCommandFailedError { stdout, stderr }
        );

        Ok(stdout)
    }
}

pub struct CopyArgs {
    pub auth_file: Option<PathBuf>,
}

impl Args for CopyArgs {
    fn args(&self) -> Result<Vec<String>> {
        let mut args: Vec<String> = Vec::new();

        if let Some(auth_file) = self.auth_file.clone() {
            let auth_file = auth_file.canonicalize().context(InvalidPathError {})?;
            args.push(String::from("--authfile"));
            args.push(String::from(auth_file.to_str().unwrap()))
        }

        Ok(args)
    }
}
