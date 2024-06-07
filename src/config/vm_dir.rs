use std::fs;
use std::path::PathBuf;

use libc::pid_t;
use tracing::info;
use uuid::Uuid;

use super::vm_config::VmConfig;
use crate::util::exception::Exception;
use crate::util::file_lock::FileLock;
use crate::util::json;
use crate::util::path::PathExtension;

pub struct VmDir {
    pub dir: PathBuf,
    pub nvram_path: PathBuf,
    pub disk_path: PathBuf,
    pub config_path: PathBuf,
}

impl VmDir {
    fn new(dir: PathBuf) -> Self {
        let nvram_path = dir.as_path().join("nvram.bin");
        let disk_path = dir.as_path().join("disk.img");
        let config_path = dir.as_path().join("config.json");
        VmDir {
            dir,
            nvram_path,
            disk_path,
            config_path,
        }
    }

    pub fn name(&self) -> String {
        self.dir.file_name().unwrap().to_string_lossy().to_string()
    }

    pub fn initialized(&self) -> bool {
        self.config_path.exists() && self.disk_path.exists() && self.nvram_path.exists()
    }

    pub fn load_config(&self) -> Result<VmConfig, Exception> {
        let json = fs::read_to_string(&self.config_path)?;
        json::from_json(&json)
    }

    pub fn save_config(&self, config: &VmConfig) -> Result<(), Exception> {
        let json = json::to_json_pretty(&config)?;
        fs::write(&self.config_path, json)?;
        Ok(())
    }

    pub fn resize(&self, size: u64) -> Result<(), Exception> {
        let file = fs::OpenOptions::new().create(true).append(true).open(&self.disk_path)?;
        file.set_len(size)?;
        Ok(())
    }

    pub fn lock(&self) -> Result<FileLock, Exception> {
        let lock = FileLock::new(&self.config_path);
        if lock.lock() {
            Ok(lock)
        } else {
            Err(Exception::ValidationError(format!("vm is already running, name={}", self.name())))
        }
    }

    pub fn pid(&self) -> Option<pid_t> {
        let lock = FileLock::new(&self.config_path);
        lock.pid()
    }
}

pub fn home_dir() -> PathBuf {
    PathBuf::from("~/.vm").to_absolute_path()
}

pub fn vm_dir(name: &str) -> VmDir {
    VmDir::new(home_dir().join(name))
}

pub fn create_temp_vm_dir() -> Result<VmDir, Exception> {
    let temp_dir = home_dir().join(Uuid::new_v4().to_string());
    info!("create temp vm dir, dir={}", temp_dir.to_string_lossy());
    fs::create_dir_all(&temp_dir)?;
    Ok(VmDir::new(temp_dir))
}
