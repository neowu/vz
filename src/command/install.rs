use std::path::PathBuf;

use clap::Args;
use clap::ValueHint;
use tracing::info;

use crate::config::vm_config::Os;
use crate::config::vm_dir;
use crate::util::exception::Exception;
use crate::vm::mac_os;
use crate::vm::mac_os_installer;

#[derive(Args)]
pub struct Install {
    #[arg(help = "vm name")]
    name: String,

    #[arg(long, help = "macOS restore image ipsw url, e.g. --ipsw=\"UniversalMac_14.5_23F79_Restore.ipsw\"", value_hint = ValueHint::FilePath)]
    ipsw: PathBuf,
}

impl Install {
    pub fn execute(&self) -> Result<(), Exception> {
        info!("instal macOS");
        let dir = vm_dir::vm_dir(&self.name);
        if !dir.initialized() {
            return Err(Exception::new(format!("vm not initialized, name={}", self.name)));
        }
        let config = dir.load_config()?;
        if !matches!(config.os, Os::MacOs) {
            return Err(Exception::new("install requires macOS guest".to_string()));
        }

        let _lock = dir.lock()?;

        let vm = mac_os::create_vm(&dir, &config)?;
        mac_os_installer::install(vm, &self.ipsw)?;

        Ok(())
    }
}
