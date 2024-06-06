use std::path::PathBuf;

use clap::Args;
use clap::ValueHint;
use tracing::info;

use crate::config::vm_config::Os;
use crate::config::vm_dir;
use crate::util::exception::Exception;
use crate::util::path::PathExtension;
use crate::vm::mac_os;
use crate::vm::mac_os_installer;

#[derive(Args)]
pub struct Install {
    #[arg(help = "vm name")]
    name: String,

    #[arg(long, help = "macOS restore image file, e.g. --ipsw=UniversalMac_14.5_23F79_Restore.ipsw", value_hint = ValueHint::FilePath)]
    ipsw: PathBuf,
}

impl Install {
    pub fn execute(&self) -> Result<(), Exception> {
        info!("instal macOS");
        let dir = vm_dir::vm_dir(&self.name);
        if !dir.initialized() {
            return Err(Exception::ValidationError(format!("vm not initialized, name={}", self.name)));
        }
        let config = dir.load_config()?;
        if !matches!(config.os, Os::MacOs) {
            return Err(Exception::ValidationError("install requires macOS guest".to_string()));
        }
        if !self.ipsw.exists() {
            return Err(Exception::ValidationError(format!(
                "ipsw does not exist, path={}",
                self.ipsw.to_string_lossy()
            )));
        }

        let _lock = dir.lock()?;

        let vm = mac_os::create_vm(&dir, &config)?;
        mac_os_installer::install(vm, &self.ipsw.to_absolute_path())?;

        Ok(())
    }
}
