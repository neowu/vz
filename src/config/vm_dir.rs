use std::fs;
use std::path::PathBuf;

use libc::pid_t;
use tracing::info;
use uuid::Uuid;

use super::vm_config::VmConfig;
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

    pub fn load_config(&self) -> VmConfig {
        let json = fs::read_to_string(&self.config_path).unwrap_or_else(|err| panic!("failed to load config, err={err}"));
        json::from_json(&json)
    }

    pub fn save_config(&self, config: &VmConfig) {
        let json = json::to_json_pretty(&config);
        fs::write(&self.config_path, json).unwrap_or_else(|err| panic!("failed to save config, err={err}"))
    }

    pub fn resize(&self, size: u64) {
        let file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.disk_path)
            .unwrap_or_else(|err| panic!("failed to open file, err={err}"));
        file.set_len(size).unwrap_or_else(|err| panic!("failed to resize file, err={err}"))
    }

    pub fn lock(&self) -> FileLock {
        let lock = FileLock::new(&self.config_path);
        if lock.lock() {
            lock
        } else {
            panic!("vm is already running, name={}", self.name())
        }
    }

    pub fn pid(&self) -> Option<pid_t> {
        let lock = FileLock::new(&self.config_path);
        lock.pid()
    }
}

pub fn home_dir() -> PathBuf {
    PathBuf::from("~/.local/share/vz").to_absolute_path()
}

pub fn vm_dir(name: &str) -> VmDir {
    VmDir::new(home_dir().join(name))
}

pub fn vm_dirs() -> Vec<VmDir> {
    if let Ok(read_dir) = home_dir().read_dir() {
        read_dir
            .into_iter()
            .flatten()
            .map(|dir| VmDir::new(dir.path()))
            .filter(|dir| dir.initialized())
            .collect()
    } else {
        vec![]
    }
}

pub fn create_temp_vm_dir() -> VmDir {
    let temp_dir = home_dir().join(Uuid::now_v7().to_string());
    info!("create temp vm dir, dir={}", temp_dir.to_string_lossy());
    fs::create_dir_all(&temp_dir).unwrap_or_else(|err| panic!("failed to create temp vm dir, err={err}"));
    VmDir::new(temp_dir)
}
