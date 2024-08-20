use anyhow::bail;
use anyhow::Result;
use clap::Args;
use log::info;

use crate::config::vm_dir;

#[derive(Args)]
pub struct Resize {
    #[arg(help = "vm name")]
    name: String,

    #[arg(long, help = "disk size in gb")]
    disk: u64,
}

impl Resize {
    pub fn execute(&self) -> Result<()> {
        let name = &self.name;
        let dir = vm_dir::vm_dir(name);
        if !dir.initialized() {
            bail!("vm not initialized, name={name}");
        }

        let size = dir.disk_path.metadata()?.len();
        if size >= self.disk * 1_000_000_000 {
            bail!("disk size must larger than current, current={size}");
        }

        info!("increase disk size, file={}, size={}G", dir.disk_path.to_string_lossy(), self.disk);
        dir.resize(self.disk * 1_000_000_000)?;
        Ok(())
    }
}
