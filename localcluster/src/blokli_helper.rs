use std::{
    fs::{self, File},
    path::Path,
    process::{Child, Command, Stdio},
};

use anyhow::{Context, Result};

pub struct ChainHandle {
    name: String,
    child: Child,
}

impl ChainHandle {
    pub fn start(chain_image: &str, log_dir: &Path) -> Result<Self> {
        fs::create_dir_all(log_dir).context("failed to create log directory")?;
        let log_file = log_dir.join("chain.log");
        let log_file = File::create(&log_file).context("failed to create blokli log file")?;
        let log_err = log_file.try_clone().context("failed to clone blokli log file handle")?;
        let name = "hopr-chain";

        let mut cmd = Command::new("docker");
        cmd.arg("run")
            .arg("--rm")
            .arg("--name")
            .arg(name)
            .arg("-p")
            .arg("8080:8080")
            .arg(chain_image)
            .stdout(Stdio::from(log_file))
            .stderr(Stdio::from(log_err));

        let child = cmd.spawn().context("failed to start blokli container")?;

        Ok(Self {
            name: name.to_string(),
            child,
        })
    }

    pub fn stop(&mut self) {
        let _ = self.child.kill();
        let _ = Command::new("docker").arg("rm").arg("-f").arg(&self.name).status();
    }
}
