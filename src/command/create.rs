use std::cmp::max;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::channel;

use block2::StackBlock;
use clap::Args;
use clap::ValueHint;
use objc2::exception::catch;
use objc2::rc::Id;
use objc2::rc::Retained;
use objc2::ClassType;
use objc2_foundation::MainThreadMarker;
use objc2_foundation::NSDataBase64EncodingOptions;
use objc2_foundation::NSError;
use objc2_virtualization::VZEFIVariableStore;
use objc2_virtualization::VZEFIVariableStoreInitializationOptions;
use objc2_virtualization::VZMACAddress;
use objc2_virtualization::VZMacAuxiliaryStorage;
use objc2_virtualization::VZMacAuxiliaryStorageInitializationOptions;
use objc2_virtualization::VZMacMachineIdentifier;
use objc2_virtualization::VZMacOSRestoreImage;
use tracing::info;

use crate::config::vm_config::Os;
use crate::config::vm_config::VmConfig;
use crate::config::vm_dir;
use crate::config::vm_dir::VmDir;
use crate::util::exception::Exception;
use crate::util::objc::ToNsUrl;
use crate::util::path::UserPath;
use crate::vm::mac_os;

#[derive(Args)]
pub struct Create {
    #[arg(help = "vm name")]
    name: String,

    #[arg(long, help = "create a linux or macOS vm", default_value = "linux")]
    os: Os,

    #[arg(long, help = "disk size in gb", default_value_t = 50)]
    disk_size: u64,

    #[arg(long, help = "macOS restore image ipsw url, e.g. --ipsw=\"UniversalMac_14.5_23F79_Restore.ipsw\"", value_hint = ValueHint::FilePath)]
    ipsw: Option<PathBuf>,
}

impl Create {
    pub fn execute(&self) -> Result<(), Exception> {
        self.validate()?;

        let temp_dir = vm_dir::create_temp_vm_dir()?;
        temp_dir.resize(self.disk_size * 1_000_000_000)?;

        let marker = MainThreadMarker::new().unwrap();

        match self.os {
            Os::Linux => create_linux(&temp_dir)?,
            Os::MacOs => create_macos(&temp_dir, &self.ipsw.as_ref().unwrap().to_absolute_path(), marker)?,
        }

        let dir = vm_dir::vm_dir(&self.name);
        info!("move vm dir, from={}, to={}", temp_dir.dir.to_string_lossy(), dir.dir.to_string_lossy());
        fs::rename(&temp_dir.dir, &dir.dir)?;
        info!("vm created, name={}, config={}", self.name, dir.config_path.to_string_lossy());

        Ok(())
    }

    fn validate(&self) -> Result<(), Exception> {
        let name = &self.name;
        let dir = vm_dir::vm_dir(name);
        if dir.initialized() {
            return Err(Exception::new(format!("vm already exists, name={name}")));
        }
        if let Os::MacOs = self.os {
            if self.ipsw.is_none() {
                return Err(Exception::new("ipsw must not be null for macOS vm".to_string()));
            }
        };
        Ok(())
    }
}

fn create_linux(dir: &VmDir) -> Result<(), Exception> {
    info!("create nvram.bin");
    unsafe {
        catch(|| {
            VZEFIVariableStore::initCreatingVariableStoreAtURL_options_error(
                VZEFIVariableStore::alloc(),
                &dir.nvram_path.to_ns_url(),
                VZEFIVariableStoreInitializationOptions::empty(),
            )
        })??;
    }

    info!("create config.json");
    let config = VmConfig {
        os: Os::Linux,
        cpu: 1,
        memory: 1024 * 1024 * 1024,
        mac_address: random_mac_address(),
        display: "1024x768".to_string(),
        sharing: HashMap::new(),
        rosetta: Some(false),
        hardware_model: None,
        machine_identifier: None,
    };
    dir.save_config(&config)?;

    Ok(())
}

fn create_macos(dir: &VmDir, ipsw: &Path, _marker: MainThreadMarker) -> Result<(), Exception> {
    let image = load_mac_os_restore_image(ipsw)?;

    let requirements = unsafe {
        image
            .mostFeaturefulSupportedConfiguration()
            .ok_or_else(|| Exception::new("restore image is not supported by current host".to_string()))
    }?;

    info!("create nvram.bin");
    let hardware_model = unsafe {
        requirements
            .hardwareModel()
            .dataRepresentation()
            .base64EncodedStringWithOptions(NSDataBase64EncodingOptions::empty())
            .to_string()
    };
    unsafe {
        let hardware_model = &hardware_model;
        catch(move || {
            let model = mac_os::hardware_model(hardware_model);
            VZMacAuxiliaryStorage::initCreatingStorageAtURL_hardwareModel_options_error(
                VZMacAuxiliaryStorage::alloc(),
                &dir.nvram_path.to_ns_url(),
                &model,
                VZMacAuxiliaryStorageInitializationOptions::empty(),
            )
        })??;
    }

    info!("create config.json");
    let machine_identifier = unsafe {
        VZMacMachineIdentifier::new()
            .dataRepresentation()
            .base64EncodedStringWithOptions(NSDataBase64EncodingOptions::empty())
            .to_string()
    };
    let config = VmConfig {
        os: Os::MacOs,
        cpu: max(4, unsafe { requirements.minimumSupportedCPUCount() }),
        memory: max(8 * 1024 * 1024 * 1024, unsafe { requirements.minimumSupportedMemorySize() }),
        mac_address: random_mac_address(),
        display: "1920x1080".to_string(),
        sharing: HashMap::new(),
        rosetta: None,
        hardware_model: Some(hardware_model),
        machine_identifier: Some(machine_identifier),
    };
    dir.save_config(&config)?;
    Ok(())
}

fn random_mac_address() -> String {
    unsafe { VZMACAddress::randomLocallyAdministeredAddress().string().to_string() }
}

fn load_mac_os_restore_image(ipsw: &Path) -> Result<Retained<VZMacOSRestoreImage>, Exception> {
    let (tx, rx) = channel();
    unsafe {
        let block = StackBlock::new(move |image: *mut VZMacOSRestoreImage, error: *mut NSError| {
            if !error.is_null() {
                tx.send(Err(Exception::new((*error).localizedDescription().to_string()))).unwrap();
            } else {
                let image = Id::from_raw(image).unwrap();
                tx.send(Ok(image)).unwrap();
            }
        });
        VZMacOSRestoreImage::loadFileURL_completionHandler(&ipsw.to_ns_url(), &block);
    };
    let image = rx.recv()??;
    Ok(image)
}
