use std::fs;
use std::path::PathBuf;

use super::vm_config::VMConfig;
use crate::util::exception::Exception;
use crate::util::json;

pub struct VMDir {
    pub dir: PathBuf,
    pub nvram_path: PathBuf,
    pub disk_path: PathBuf,
    pub config_path: PathBuf,
}

impl VMDir {
    fn new(dir: PathBuf) -> Self {
        let nvram_path = dir.as_path().join("nvram.bin");
        let disk_path = dir.as_path().join("disk.img");
        let config_path = dir.as_path().join("config.json");
        VMDir {
            dir,
            nvram_path,
            disk_path,
            config_path,
        }
    }

    fn name(&self) -> String {
        self.dir.file_name().unwrap().to_string_lossy().to_string()
    }

    pub fn initialized(&self) -> bool {
        self.config_path.exists() && self.disk_path.exists() && self.nvram_path.exists()
    }

    pub fn load_config(&self) -> Result<VMConfig, Exception> {
        let json = fs::read_to_string(&self.config_path)?;
        json::from_json(&json)
    }
}

pub fn vm_dir(name: &str) -> VMDir {
    let home = env!("HOME");
    VMDir::new(PathBuf::from(format!("{home}/.vm/{name}")))
}
