use std::path::Path;
use std::path::PathBuf;

use objc2::exception::catch;
use objc2::rc::Id;
use objc2::rc::Retained;
use objc2::ClassType;
use objc2_foundation::ns_string;
use objc2_foundation::NSArray;
use objc2_foundation::NSString;
use objc2_foundation::NSURL;
use objc2_virtualization::VZDirectorySharingDeviceConfiguration;
use objc2_virtualization::VZDiskImageCachingMode;
use objc2_virtualization::VZDiskImageStorageDeviceAttachment;
use objc2_virtualization::VZDiskImageSynchronizationMode;
use objc2_virtualization::VZEFIBootLoader;
use objc2_virtualization::VZEFIVariableStore;
use objc2_virtualization::VZGenericPlatformConfiguration;
use objc2_virtualization::VZGraphicsDeviceConfiguration;
use objc2_virtualization::VZLinuxRosettaDirectoryShare;
use objc2_virtualization::VZStorageDeviceConfiguration;
use objc2_virtualization::VZUSBKeyboardConfiguration;
use objc2_virtualization::VZUSBMassStorageDeviceConfiguration;
use objc2_virtualization::VZUSBScreenCoordinatePointingDeviceConfiguration;
use objc2_virtualization::VZVirtioBlockDeviceConfiguration;
use objc2_virtualization::VZVirtioEntropyDeviceConfiguration;
use objc2_virtualization::VZVirtioFileSystemDeviceConfiguration;
use objc2_virtualization::VZVirtioGraphicsDeviceConfiguration;
use objc2_virtualization::VZVirtioGraphicsScanoutConfiguration;
use objc2_virtualization::VZVirtioTraditionalMemoryBalloonDeviceConfiguration;
use objc2_virtualization::VZVirtualMachine;
use objc2_virtualization::VZVirtualMachineConfiguration;
use tracing::info;

use crate::config::vm_config::VmConfig;
use crate::config::vm_dir::VmDir;
use crate::util::exception::Exception;
use crate::util::path::PathExtension;

pub fn create_vm(dir: &VmDir, config: &VmConfig, gui: bool, mount: Option<&PathBuf>) -> Result<Retained<VZVirtualMachine>, Exception> {
    info!("create linux vm, name={}", dir.name());
    let vz_config = create_vm_config(dir, config, gui, mount)?;
    unsafe {
        vz_config.validateWithError()?;
        Ok(VZVirtualMachine::initWithConfiguration(VZVirtualMachine::alloc(), &vz_config))
    }
}

fn create_vm_config(
    dir: &VmDir,
    config: &VmConfig,
    gui: bool,
    mount: Option<&PathBuf>,
) -> Result<Retained<VZVirtualMachineConfiguration>, Exception> {
    unsafe {
        let vz_config = VZVirtualMachineConfiguration::new();
        vz_config.setCPUCount(config.cpu);
        vz_config.setMemorySize(config.memory);

        vz_config.setBootLoader(Option::Some(&boot_loader(dir)));
        vz_config.setPlatform(&VZGenericPlatformConfiguration::new());

        if gui {
            vz_config.setGraphicsDevices(&NSArray::from_vec(vec![display(1024, 768)]));
            vz_config.setKeyboards(&NSArray::from_vec(vec![Id::into_super(VZUSBKeyboardConfiguration::new())]));
            vz_config.setPointingDevices(&NSArray::from_vec(vec![Id::into_super(
                VZUSBScreenCoordinatePointingDeviceConfiguration::new(),
            )]));
        }

        vz_config.setNetworkDevices(&NSArray::from_vec(vec![config.network()]));
        vz_config.setStorageDevices(&NSArray::from_vec(storage(dir, mount)?));

        vz_config.setMemoryBalloonDevices(&NSArray::from_vec(vec![Id::into_super(
            VZVirtioTraditionalMemoryBalloonDeviceConfiguration::new(),
        )]));
        vz_config.setEntropyDevices(&NSArray::from_vec(vec![Id::into_super(VZVirtioEntropyDeviceConfiguration::new())]));

        let mut sharings: Vec<Retained<VZDirectorySharingDeviceConfiguration>> = vec![];
        if let Some(sharing) = config.sharing_directories()? {
            sharings.push(sharing);
        }
        if let Some(true) = config.rosetta {
            let device = VZVirtioFileSystemDeviceConfiguration::initWithTag(VZVirtioFileSystemDeviceConfiguration::alloc(), ns_string!("rosetta"));
            device.setShare(Some(&Id::into_super(VZLinuxRosettaDirectoryShare::new())));
            sharings.push(Id::into_super(device));
        }
        vz_config.setDirectorySharingDevices(&NSArray::from_vec(sharings));

        Ok(vz_config)
    }
}

fn boot_loader(dir: &VmDir) -> Retained<VZEFIBootLoader> {
    unsafe {
        let store = VZEFIVariableStore::initWithURL(VZEFIVariableStore::alloc(), &dir.nvram_path.to_ns_url());
        let loader = VZEFIBootLoader::new();
        loader.setVariableStore(Option::Some(&store));
        loader
    }
}

fn storage(dir: &VmDir, mount: Option<&PathBuf>) -> Result<Vec<Retained<VZStorageDeviceConfiguration>>, Exception> {
    let disk = disk(&dir.disk_path)?;
    let mut storage = vec![disk];
    if let Option::Some(mount) = mount {
        let disk = mount_disk(mount)?;
        storage.push(disk)
    }
    Ok(storage)
}

fn disk(disk: &Path) -> Result<Retained<VZStorageDeviceConfiguration>, Exception> {
    unsafe {
        let attachment = catch(|| {
            let url = NSURL::initFileURLWithPath(NSURL::alloc(), &NSString::from_str(&disk.to_string_lossy()));
            VZDiskImageStorageDeviceAttachment::initWithURL_readOnly_cachingMode_synchronizationMode_error(
                VZDiskImageStorageDeviceAttachment::alloc(),
                &url,
                false,
                VZDiskImageCachingMode::Automatic,
                VZDiskImageSynchronizationMode::Fsync,
            )
        })??;
        let disk = VZVirtioBlockDeviceConfiguration::initWithAttachment(VZVirtioBlockDeviceConfiguration::alloc(), &attachment);
        Ok(Id::into_super(disk))
    }
}

fn mount_disk(mount: &Path) -> Result<Retained<VZStorageDeviceConfiguration>, Exception> {
    unsafe {
        let attachment = catch(|| {
            VZDiskImageStorageDeviceAttachment::initWithURL_readOnly_error(VZDiskImageStorageDeviceAttachment::alloc(), &mount.to_ns_url(), true)
        })??;
        let disk = VZUSBMassStorageDeviceConfiguration::initWithAttachment(VZUSBMassStorageDeviceConfiguration::alloc(), &attachment);
        Ok(Id::into_super(disk))
    }
}

fn display(width: isize, height: isize) -> Retained<VZGraphicsDeviceConfiguration> {
    unsafe {
        let display = VZVirtioGraphicsDeviceConfiguration::new();
        let scanout =
            VZVirtioGraphicsScanoutConfiguration::initWithWidthInPixels_heightInPixels(VZVirtioGraphicsScanoutConfiguration::alloc(), width, height);
        let scanouts = &NSArray::from_vec(vec![scanout]);
        display.setScanouts(scanouts);
        Id::into_super(display)
    }
}
