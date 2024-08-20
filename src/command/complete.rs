use anyhow::Result;
use clap::Args;

use crate::config::vm_dir;

#[derive(Args)]
pub struct Complete {
    #[arg()]
    name: String,
}

impl Complete {
    pub fn execute(&self) -> Result<()> {
        if self.name == "vm_name" {
            for vm_dir in vm_dir::vm_dirs().into_iter() {
                println!("{}\tvm", vm_dir.name());
            }
        }
        Ok(())
    }
}
