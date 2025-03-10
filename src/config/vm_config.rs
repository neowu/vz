use std::collections::HashMap;
use std::path::PathBuf;

use objc2::AllocAnyThread;
use objc2::rc::Retained;
use objc2_foundation::NSDictionary;
use objc2_foundation::NSString;
use objc2_virtualization::VZDirectorySharingDeviceConfiguration;
use objc2_virtualization::VZMACAddress;
use objc2_virtualization::VZMultipleDirectoryShare;
use objc2_virtualization::VZNATNetworkDeviceAttachment;
use objc2_virtualization::VZNetworkDeviceConfiguration;
use objc2_virtualization::VZSharedDirectory;
use objc2_virtualization::VZVirtioFileSystemDeviceConfiguration;
use objc2_virtualization::VZVirtioNetworkDeviceConfiguration;
use serde::Deserialize;
use serde::Serialize;

use crate::util::path::PathExtension;

#[derive(Serialize, Deserialize, Debug, Clone, clap::ValueEnum)]
pub enum Os {
    #[serde(rename = "linux")]
    #[clap(name = "linux")]
    Linux,
    #[serde(rename = "macOS")]
    #[clap(name = "macOS")]
    MacOs,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VmConfig {
    pub os: Os,
    pub cpu: usize,
    pub memory: u64,
    #[serde(rename = "macAddress")]
    pub mac_address: String,
    pub sharing: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rosetta: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hardware_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub machine_identifier: Option<String>,
}

impl VmConfig {
    pub fn network(&self) -> Retained<VZNetworkDeviceConfiguration> {
        unsafe {
            let network = VZVirtioNetworkDeviceConfiguration::new();
            network.setAttachment(Some(&VZNATNetworkDeviceAttachment::new()));
            let mac_address = VZMACAddress::initWithString(VZMACAddress::alloc(), &NSString::from_str(&self.mac_address)).unwrap();
            network.setMACAddress(&mac_address);
            Retained::into_super(network)
        }
    }

    pub fn sharing_directories(&self) -> Option<Retained<VZDirectorySharingDeviceConfiguration>> {
        if self.sharing.is_empty() {
            return None;
        }
        let mut keys: Vec<Retained<NSString>> = vec![];
        let mut values: Vec<Retained<VZSharedDirectory>> = vec![];

        for (key, value) in self.sharing.iter() {
            keys.push(NSString::from_str(key));
            let path = PathBuf::from(value).to_absolute_path();
            if !path.exists() {
                panic!("sharing path does not exist, name={key}, path={}", path.to_string_lossy());
            }
            unsafe {
                values.push(VZSharedDirectory::initWithURL_readOnly(
                    VZSharedDirectory::alloc(),
                    &path.to_ns_url(),
                    false,
                ));
            }
        }

        let keys: Vec<&NSString> = keys.iter().map(|v| v.as_ref()).collect();
        let directories = NSDictionary::from_retained_objects(&keys, &values);
        unsafe {
            let device = VZVirtioFileSystemDeviceConfiguration::initWithTag(
                VZVirtioFileSystemDeviceConfiguration::alloc(),
                &VZVirtioFileSystemDeviceConfiguration::macOSGuestAutomountTag(),
            );
            let sharings = VZMultipleDirectoryShare::initWithDirectories(VZMultipleDirectoryShare::alloc(), &directories);
            device.setShare(Some(&Retained::into_super(sharings)));
            Some(Retained::into_super(device))
        }
    }
}
