use clap::Args;
use tracing::info;

use crate::config::vm_dir;
use crate::util::exception::Exception;

#[derive(Args)]
pub struct Resize {
    #[arg(help = "vm name")]
    name: String,

    #[arg(long, help = "disk size in gb", default_value_t = 50)]
    disk_size: u64,
}

impl Resize {
    pub fn execute(&self) -> Result<(), Exception> {
        let name = &self.name;
        let dir = vm_dir::vm_dir(name);
        if !dir.initialized() {
            return Err(Exception::ValidationError(format!("vm not initialized, name={name}")));
        }

        let size = dir.disk_path.metadata()?.len();
        if size >= self.disk_size * 1_000_000_000 {
            return Err(Exception::ValidationError(format!("disk size must larger than current, current={size}")));
        }

        info!("increase disk size, file={}, size={}G", dir.disk_path.to_string_lossy(), self.disk_size);
        dir.resize(self.disk_size * 1_000_000_000)?;
        Ok(())
    }
}
