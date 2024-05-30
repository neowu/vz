use std::collections::HashMap;
use std::path::PathBuf;

use clap::command;
use clap::Args;
use clap::ValueHint;
use objc2::exception::catch;
use objc2::ClassType;
use objc2_virtualization::VZEFIVariableStore;
use objc2_virtualization::VZEFIVariableStoreInitializationOptions;
use objc2_virtualization::VZMACAddress;
use tokio::fs;
use tracing::info;

use crate::config::vm_config::VMConfig;
use crate::config::vm_config::OS;
use crate::config::vm_dir;
use crate::config::vm_dir::VMDir;
use crate::util::exception::Exception;
use crate::util::objc;

#[derive(Args)]
#[command(about = "create vm")]
pub struct Create {
    #[arg(long, help = "vm name")]
    name: String,

    #[arg(long, help = "create a linux or macOS vm", default_value = "linux")]
    os: OS,

    #[arg(long, help = "disk size in gb", default_value_t = 50)]
    disk_size: u64,

    #[arg(long, help = "macOS restore image ipsw url, e.g. --ipsw=\"UniversalMac_14.1.1_23B81_Restore.ipsw\"", value_hint = ValueHint::FilePath)]
    ipsw: Option<PathBuf>,
}

impl Create {
    pub async fn execute(&self) -> Result<(), Exception> {
        self.validate()?;

        let temp_vm_dir = vm_dir::create_temp_vm_dir().await?;
        temp_vm_dir.resize(self.disk_size * 1_000_000_000).await?;

        match self.os {
            OS::Linux => create_linux(&temp_vm_dir).await?,
            OS::MacOS => todo!(),
        }

        let vm_dir = vm_dir::vm_dir(&self.name);
        info!(
            "move vm dir, from={}, to={}",
            temp_vm_dir.dir.to_string_lossy(),
            vm_dir.dir.to_string_lossy()
        );
        fs::rename(&temp_vm_dir.dir, &vm_dir.dir).await?;
        info!("vm created, name={}, config={}", self.name, vm_dir.config_path.to_string_lossy());

        Ok(())
    }

    fn validate(&self) -> Result<(), Exception> {
        let name = &self.name;
        let vm_dir = vm_dir::vm_dir(name);
        if vm_dir.initialized() {
            return Err(Exception::new(format!("vm already exists, name={name}")));
        }
        if let OS::MacOS = self.os {
            if self.ipsw.is_none() {
                return Err(Exception::new("ipsw must not be null for macOS vm".to_string()));
            }
        };
        Ok(())
    }
}

async fn create_linux(dir: &VMDir) -> Result<(), Exception> {
    info!("create nvram.bin");
    unsafe {
        catch(|| {
            VZEFIVariableStore::initCreatingVariableStoreAtURL_options_error(
                VZEFIVariableStore::alloc(),
                &objc::file_url(&dir.nvram_path),
                VZEFIVariableStoreInitializationOptions::empty(),
            )
        })??;
    }

    info!("create config.json");
    let mac_address;
    unsafe {
        mac_address = VZMACAddress::randomLocallyAdministeredAddress().string().to_string();
    }
    let config = VMConfig {
        os: OS::Linux,
        cpu: 1,
        memory: 1024 * 1024 * 1024,
        mac_address,
        display: "1024x768".to_string(),
        sharing: HashMap::new(),
        rosetta: Some(false),
    };
    dir.save_config(config).await?;

    Ok(())
}
