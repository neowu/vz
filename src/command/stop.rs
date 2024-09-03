use std::process;
use std::thread::sleep;
use std::time::Duration;

use clap::Args;
use log::error;
use log::info;

use crate::config::vm_dir;
use crate::config::vm_dir::VmDir;

#[derive(Args)]
pub struct Stop {
    #[arg(help = "vm name")]
    name: String,
}

impl Stop {
    pub fn execute(&self) {
        let name = &self.name;
        let dir = vm_dir::vm_dir(name);
        if !dir.initialized() {
            panic!("vm not initialized, name={name}");
        }

        let pid = dir.pid().unwrap_or_else(|| panic!("vm not running, name={name}"));
        info!("stop vm, name={name}, pid={pid}");
        unsafe {
            libc::kill(pid, libc::SIGINT);
        }

        let success = wait_until_stopped(dir);
        if success {
            info!("vm stopped");
            process::exit(0);
        } else {
            error!("failed to stop vm");
            process::exit(1);
        }
    }
}

fn wait_until_stopped(dir: VmDir) -> bool {
    let mut attempts = 0;
    while attempts < 20 {
        sleep(Duration::from_secs(1));
        if dir.pid().is_none() {
            return true;
        }
        attempts += 1;
    }
    false
}
