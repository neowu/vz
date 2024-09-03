use std::path::PathBuf;

use clap::Args;
use clap::ValueHint;
use log::info;
use objc2_foundation::MainThreadMarker;

use crate::config::vm_config::Os;
use crate::config::vm_dir;
use crate::util::path::PathExtension;
use crate::vm::mac_os;
use crate::vm::mac_os_installer;

#[derive(Args)]
pub struct Install {
    #[arg(help = "vm name")]
    name: String,

    #[arg(long, help = "macOS restore image file, e.g. --ipsw=UniversalMac_14.5_23F79_Restore.ipsw", value_hint = ValueHint::FilePath)]
    ipsw: PathBuf,
}

impl Install {
    pub fn execute(&self) {
        self.validate();

        let name = &self.name;
        let dir = vm_dir::vm_dir(name);
        if !dir.initialized() {
            panic!("vm not initialized, name={name}");
        }
        let config = dir.load_config();
        if !matches!(config.os, Os::MacOs) {
            panic!("install requires macOS guest");
        }
        let _lock = dir.lock();

        info!("instal macOS");
        let marker = MainThreadMarker::new().unwrap();
        let vm = mac_os::create_vm(&dir, &config, marker);
        mac_os_installer::install(vm, &self.ipsw.to_absolute_path(), marker);
    }

    fn validate(&self) {
        if !self.ipsw.exists() {
            panic!("ipsw does not exist, path={}", self.ipsw.to_string_lossy());
        }
    }
}
