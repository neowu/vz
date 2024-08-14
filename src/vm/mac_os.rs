use std::path::Path;

use anyhow::Result;
use log::info;
use objc2::exception::catch;
use objc2::rc::Id;
use objc2::rc::Retained;
use objc2::ClassType;
use objc2_app_kit::NSScreen;
use objc2_foundation::CGFloat;
use objc2_foundation::MainThreadMarker;
use objc2_foundation::NSArray;
use objc2_foundation::NSData;
use objc2_foundation::NSDataBase64DecodingOptions;
use objc2_foundation::NSSize;
use objc2_foundation::NSString;
use objc2_virtualization::VZDiskImageCachingMode;
use objc2_virtualization::VZDiskImageStorageDeviceAttachment;
use objc2_virtualization::VZDiskImageSynchronizationMode;
use objc2_virtualization::VZGraphicsDeviceConfiguration;
use objc2_virtualization::VZMacAuxiliaryStorage;
use objc2_virtualization::VZMacGraphicsDeviceConfiguration;
use objc2_virtualization::VZMacGraphicsDisplayConfiguration;
use objc2_virtualization::VZMacHardwareModel;
use objc2_virtualization::VZMacKeyboardConfiguration;
use objc2_virtualization::VZMacMachineIdentifier;
use objc2_virtualization::VZMacOSBootLoader;
use objc2_virtualization::VZMacPlatformConfiguration;
use objc2_virtualization::VZMacTrackpadConfiguration;
use objc2_virtualization::VZPlatformConfiguration;
use objc2_virtualization::VZStorageDeviceConfiguration;
use objc2_virtualization::VZVirtioBlockDeviceConfiguration;
use objc2_virtualization::VZVirtioEntropyDeviceConfiguration;
use objc2_virtualization::VZVirtioTraditionalMemoryBalloonDeviceConfiguration;
use objc2_virtualization::VZVirtualMachine;
use objc2_virtualization::VZVirtualMachineConfiguration;

use crate::config::vm_config::VmConfig;
use crate::config::vm_dir::VmDir;
use crate::util::objc::ObjcError;
use crate::util::path::PathExtension;

pub fn create_vm(dir: &VmDir, config: &VmConfig, marker: MainThreadMarker) -> Result<Retained<VZVirtualMachine>> {
    info!("create macOS vm, name={}", dir.name());
    let vz_config = create_vm_config(dir, config, marker)?;
    unsafe {
        vz_config.validateWithError()?;
        Ok(VZVirtualMachine::initWithConfiguration(VZVirtualMachine::alloc(), &vz_config))
    }
}

pub fn hardware_model(base64_string: &str) -> Retained<VZMacHardwareModel> {
    unsafe {
        let data_representation =
            NSData::initWithBase64EncodedString_options(NSData::alloc(), &NSString::from_str(base64_string), NSDataBase64DecodingOptions::empty())
                .unwrap();

        VZMacHardwareModel::initWithDataRepresentation(VZMacHardwareModel::alloc(), &data_representation).unwrap()
    }
}

fn create_vm_config(dir: &VmDir, config: &VmConfig, marker: MainThreadMarker) -> Result<Retained<VZVirtualMachineConfiguration>> {
    unsafe {
        let vz_config = VZVirtualMachineConfiguration::new();
        vz_config.setCPUCount(config.cpu);
        vz_config.setMemorySize(config.memory);

        vz_config.setBootLoader(Some(&VZMacOSBootLoader::new()));
        vz_config.setPlatform(&platform(dir, config));

        vz_config.setGraphicsDevices(&NSArray::from_vec(vec![display(1920, 1080, marker)]));
        vz_config.setKeyboards(&NSArray::from_vec(vec![Id::into_super(VZMacKeyboardConfiguration::new())]));
        vz_config.setPointingDevices(&NSArray::from_vec(vec![Id::into_super(VZMacTrackpadConfiguration::new())]));

        vz_config.setNetworkDevices(&NSArray::from_vec(vec![config.network()]));
        vz_config.setStorageDevices(&NSArray::from_vec(vec![disk(&dir.disk_path)?]));

        vz_config.setMemoryBalloonDevices(&NSArray::from_vec(vec![Id::into_super(
            VZVirtioTraditionalMemoryBalloonDeviceConfiguration::new(),
        )]));
        vz_config.setEntropyDevices(&NSArray::from_vec(vec![Id::into_super(VZVirtioEntropyDeviceConfiguration::new())]));

        if let Some(sharing) = config.sharing_directories()? {
            vz_config.setDirectorySharingDevices(&NSArray::from_vec(vec![sharing]));
        }
        Ok(vz_config)
    }
}

fn platform(dir: &VmDir, config: &VmConfig) -> Retained<VZPlatformConfiguration> {
    unsafe {
        let platform = VZMacPlatformConfiguration::new();
        platform.setAuxiliaryStorage(Some(&VZMacAuxiliaryStorage::initWithURL(
            VZMacAuxiliaryStorage::alloc(),
            &dir.nvram_path.to_ns_url(),
        )));
        platform.setHardwareModel(&hardware_model(config.hardware_model.as_ref().unwrap()));
        platform.setMachineIdentifier(&machine_identifier(config.machine_identifier.as_ref().unwrap()));
        Id::into_super(platform)
    }
}

fn disk(disk: &Path) -> Result<Retained<VZStorageDeviceConfiguration>> {
    unsafe {
        let attachment = catch(|| {
            VZDiskImageStorageDeviceAttachment::initWithURL_readOnly_cachingMode_synchronizationMode_error(
                VZDiskImageStorageDeviceAttachment::alloc(),
                &disk.to_ns_url(),
                false,
                VZDiskImageCachingMode::Automatic,
                VZDiskImageSynchronizationMode::Fsync,
            )
        })
        .map_err(ObjcError::from)??;
        let disk = VZVirtioBlockDeviceConfiguration::initWithAttachment(VZVirtioBlockDeviceConfiguration::alloc(), &attachment);
        Ok(Id::into_super(disk))
    }
}

fn display(width: isize, height: isize, marker: MainThreadMarker) -> Retained<VZGraphicsDeviceConfiguration> {
    let screen = NSScreen::mainScreen(marker).unwrap();
    unsafe {
        let display = VZMacGraphicsDeviceConfiguration::new();
        display.setDisplays(&NSArray::from_vec(vec![VZMacGraphicsDisplayConfiguration::initForScreen_sizeInPoints(
            VZMacGraphicsDisplayConfiguration::alloc(),
            &screen,
            NSSize::new(width as CGFloat, height as CGFloat),
        )]));
        Id::into_super(display)
    }
}

fn machine_identifier(base64_string: &str) -> Retained<VZMacMachineIdentifier> {
    unsafe {
        let data_representation =
            NSData::initWithBase64EncodedString_options(NSData::alloc(), &NSString::from_str(base64_string), NSDataBase64DecodingOptions::empty())
                .unwrap();
        VZMacMachineIdentifier::initWithDataRepresentation(VZMacMachineIdentifier::alloc(), &data_representation).unwrap()
    }
}
