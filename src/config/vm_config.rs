use std::collections::HashMap;
use std::num::ParseIntError;

use objc2::rc::Id;
use objc2::rc::Retained;
use objc2::ClassType;
use objc2_foundation::NSString;
use objc2_virtualization::VZMACAddress;
use objc2_virtualization::VZNATNetworkDeviceAttachment;
use objc2_virtualization::VZNetworkDeviceConfiguration;
use objc2_virtualization::VZVirtioNetworkDeviceConfiguration;
use serde::Deserialize;
use serde::Serialize;

use crate::util::exception::Exception;

#[derive(Serialize, Deserialize, Debug, Clone, clap::ValueEnum)]
pub enum OS {
    #[serde(rename = "linux")]
    #[clap(name = "linux")]
    Linux,
    #[serde(rename = "macOS")]
    #[clap(name = "macOS")]
    MacOS,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VMConfig {
    pub os: OS,
    pub cpu: usize,
    pub memory: u64,
    #[serde(rename = "macAddress")]
    pub mac_address: String,
    pub display: String,
    pub sharing: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rosetta: Option<bool>,
}

impl VMConfig {
    pub fn network(&self) -> Retained<VZNetworkDeviceConfiguration> {
        unsafe {
            let network = VZVirtioNetworkDeviceConfiguration::new();
            network.setAttachment(Some(&VZNATNetworkDeviceAttachment::new()));
            let mac_address = VZMACAddress::initWithString(VZMACAddress::alloc(), &NSString::from_str(&self.mac_address));
            network.setMACAddress(mac_address.unwrap().as_ref());
            Id::into_super(network)
        }
    }

    pub fn display(&self) -> Result<(isize, isize), Exception> {
        let components = self.display.split_once('x').unwrap();
        let width = components.0.parse()?;
        let height = components.1.parse()?;
        Ok((width, height))
    }
}

impl From<ParseIntError> for Exception {
    fn from(err: ParseIntError) -> Self {
        Exception::new(err.to_string())
    }
}