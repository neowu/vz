use std::fs;
use std::os::unix::fs::MetadataExt;

use clap::Args;

use crate::config::vm_dir;
use crate::util::exception::Exception;
use crate::util::json;

#[derive(Args)]
pub struct List;

impl List {
    pub fn execute(&self) -> Result<(), Exception> {
        let home_dir = vm_dir::home_dir();
        if !home_dir.exists() {
            return Err(Exception::ValidationError(format!("{} does not exist", home_dir.to_string_lossy())));
        }
        println!("{:<16}{:<8}{:<8}{:<8}{:<16}{:<16}", "name", "os", "cpu", "memory", "disk", "status");
        for entry in fs::read_dir(home_dir)? {
            let path = entry?.path();
            if path.is_dir() {
                let dir = vm_dir::vm_dir(&path.file_name().unwrap().to_string_lossy());
                if dir.initialized() {
                    let name = dir.name();

                    let config = dir.load_config()?;
                    let os = json::to_json_value(&config.os)?;
                    let cpu = config.cpu;
                    let memory = format!("{:.2}G", config.memory as f32 / (1024.0 * 1024.0 * 1024.0));
                    let metadata = dir.disk_path.metadata()?;
                    let disk = format!(
                        "{:0.2}G/{:.2}G",
                        metadata.blocks() as f32 * 512.0 / 1_000_000_000.0,
                        metadata.len() as f32 / 1_000_000_000.0
                    );
                    let status = if dir.pid().is_some() { "running" } else { "stopped" };
                    println!("{:<16}{:<8}{:<8}{:<8}{:<16}{:<16}", name, os, cpu, memory, disk, status)
                }
            }
        }

        Ok(())
    }
}
