use clap::Args;
use tracing::info;

use crate::config::vm_dir;

#[derive(Args)]
pub struct Edit {
    #[arg(help = "vm name")]
    name: String,

    #[arg(long, help = "disk size in gb")]
    disk: Option<u64>,

    #[arg(long, help = "cpu count")]
    cpu: Option<usize>,

    #[arg(long, help = "ram size in gb")]
    ram: Option<u64>,
}

impl Edit {
    pub fn execute(&self) {
        let name = &self.name;
        let dir = vm_dir::vm_dir(name);
        if !dir.initialized() {
            panic!("vm not initialized, name={name}");
        }

        // Handle disk resize
        if let Some(disk) = self.disk {
            let size = dir
                .disk_path
                .metadata()
                .unwrap_or_else(|err| panic!("failed to get metadata, err={err}"))
                .len();
            if size >= disk * 1_000_000_000 {
                panic!("disk size must larger than current, current={size}");
            }

            info!("increase disk size, file={}, size={}G", dir.disk_path.to_string_lossy(), disk);
            dir.resize(disk * 1_000_000_000);
        }

        // Handle CPU/RAM changes
        if self.cpu.is_some() || self.ram.is_some() {
            let mut config = dir.load_config();
            
            if let Some(cpu) = self.cpu {
                info!("change cpu count, current={}, new={}", config.cpu, cpu);
                config.cpu = cpu;
            }
            
            if let Some(ram) = self.ram {
                info!("change ram size, current={}G, new={}G", config.memory / 1_000_000_000, ram);
                config.memory = ram * 1_000_000_000;
            }
            
            dir.save_config(&config);
        }

        // Check if at least one argument was provided
        if self.disk.is_none() && self.cpu.is_none() && self.ram.is_none() {
            panic!("at least one of --disk, --cpu, or --ram must be specified");
        }
    }
}
