use std::path::PathBuf;

use tokio::fs;
use tokio::fs::create_dir_all;
use tokio::fs::OpenOptions;
use tracing::info;
use uuid::Uuid;

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

    pub async fn load_config(&self) -> Result<VMConfig, Exception> {
        let json = fs::read_to_string(&self.config_path).await?;
        json::from_json(&json)
    }

    pub async fn save_config(&self, config: VMConfig) -> Result<(), Exception> {
        let json = json::to_json_pretty(&config)?;
        fs::write(&self.config_path, json).await?;
        Ok(())
    }

    pub async fn resize(&self, size: u64) -> Result<(), Exception> {
        let file = OpenOptions::new().create(true).append(true).open(&self.disk_path).await?;
        file.set_len(size).await?;
        Ok(())
    }
}

pub fn vm_dir(name: &str) -> VMDir {
    let home = env!("HOME");
    VMDir::new(PathBuf::from(format!("{home}/.vm/{name}")))
}

pub async fn create_temp_vm_dir() -> Result<VMDir, Exception> {
    let home = env!("HOME");
    let temp_dir = PathBuf::from(format!("{home}/.vm/{}", Uuid::new_v4()));
    info!("create dir, dir={}", temp_dir.to_string_lossy());
    create_dir_all(&temp_dir).await?;
    Ok(VMDir::new(temp_dir))
}
