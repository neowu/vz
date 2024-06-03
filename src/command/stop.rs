use std::time::Duration;

use clap::Args;
use tokio::time::sleep;
use tracing::info;

use crate::config::vm_dir::VmDir;
use crate::config::vm_dir::{self};
use crate::util::exception::Exception;

#[derive(Args)]
pub struct Stop {
    #[arg(help = "VM name")]
    name: String,
}

impl Stop {
    pub async fn execute(&self) -> Result<(), Exception> {
        let name = &self.name;
        let vm_dir = vm_dir::vm_dir(name);
        if !vm_dir.initialized() {
            return Err(Exception::new(format!("vm not initialized, name={name}")));
        }

        let pid = vm_dir.pid().ok_or_else(|| Exception::new(format!("vm not running, name={name}")))?;
        info!("stop vm, name={name}, pid={pid}");
        unsafe {
            libc::kill(pid, libc::SIGINT);
        }

        wait_until_stopped(vm_dir).await
    }
}

async fn wait_until_stopped(dir: VmDir) -> Result<(), Exception> {
    let mut attempts = 0;
    while attempts < 20 {
        sleep(Duration::from_secs(1)).await;
        if dir.pid().is_none() {
            info!("vm stopped");
            return Ok(());
        }
        attempts += 1;
    }
    Err(Exception::new("failed to stop vm".to_string()))
}
