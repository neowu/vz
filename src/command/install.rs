use std::path::PathBuf;

use clap::Args;
use clap::ValueHint;
use tracing::info;

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
    pub async fn execute(&self) -> Result<(), Exception> {
        info!("instal macOS");
        let dir = vm_dir::vm_dir(&self.name);
        let config = dir.load_config().await?;
        let _lock = dir.lock()?;

        let vm = mac_os::create_vm(&dir, &config)?;
        mac_os_installer::install(vm, &self.ipsw)?;

        Ok(())
    }
}
