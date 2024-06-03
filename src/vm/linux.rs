use std::path::Path;
use std::path::PathBuf;

use objc2::exception::catch;
use objc2::rc::Id;
use objc2::rc::Retained;
use objc2::ClassType;
use objc2_foundation::NSArray;
use objc2_foundation::NSString;
use objc2_foundation::NSURL;
use objc2_virtualization::VZDiskImageCachingMode;
use objc2_virtualization::VZDiskImageStorageDeviceAttachment;
use objc2_virtualization::VZDiskImageSynchronizationMode;
use objc2_virtualization::VZEFIBootLoader;
use objc2_virtualization::VZEFIVariableStore;
use objc2_virtualization::VZGenericPlatformConfiguration;
use objc2_virtualization::VZGraphicsDeviceConfiguration;
use objc2_virtualization::VZStorageDeviceConfiguration;
use objc2_virtualization::VZUSBKeyboardConfiguration;
use objc2_virtualization::VZUSBMassStorageDeviceConfiguration;
use objc2_virtualization::VZUSBScreenCoordinatePointingDeviceConfiguration;
use objc2_virtualization::VZVirtioBlockDeviceConfiguration;
use objc2_virtualization::VZVirtioEntropyDeviceConfiguration;
use objc2_virtualization::VZVirtioGraphicsDeviceConfiguration;
use objc2_virtualization::VZVirtioGraphicsScanoutConfiguration;
use objc2_virtualization::VZVirtioTraditionalMemoryBalloonDeviceConfiguration;
use objc2_virtualization::VZVirtualMachine;
use objc2_virtualization::VZVirtualMachineConfiguration;
use tracing::info;

use crate::config::vm_config::VmConfig;
use crate::config::vm_dir::VmDir;
use crate::util::exception::Exception;

pub struct Linux {
    dir: VmDir,
    config: VmConfig,
    gui: bool,
    mount: Option<PathBuf>,
}

impl Linux {
    pub fn new(dir: VmDir, config: VmConfig, gui: bool, mount: Option<PathBuf>) -> Self {
        Linux { dir, config, gui, mount }
    }

    pub fn create_vm(&self) -> Result<Retained<VZVirtualMachine>, Exception> {
        info!("create linux vm, name={}", self.dir.name());
        let vz_config = self.create_vm_config()?;
        unsafe {
            vz_config.validateWithError()?;
            Ok(VZVirtualMachine::initWithConfiguration(VZVirtualMachine::alloc(), &vz_config))
        }
    }

    fn create_vm_config(&self) -> Result<Retained<VZVirtualMachineConfiguration>, Exception> {
        unsafe {
            let config = VZVirtualMachineConfiguration::new();
            config.setCPUCount(self.config.cpu);
            config.setMemorySize(self.config.memory);

            config.setBootLoader(Option::Some(self.boot_loader().as_ref()));
            config.setPlatform(VZGenericPlatformConfiguration::new().as_ref());

            if self.gui {
                let (width, height) = self.config.display()?;
                config.setGraphicsDevices(&NSArray::from_vec(vec![display(width, height)]));
                config.setKeyboards(&NSArray::from_vec(vec![Id::into_super(VZUSBKeyboardConfiguration::new())]));
                config.setPointingDevices(&NSArray::from_vec(vec![Id::into_super(
                    VZUSBScreenCoordinatePointingDeviceConfiguration::new(),
                )]));
            }

            config.setNetworkDevices(&NSArray::from_vec(vec![self.config.network()]));
            config.setStorageDevices(&NSArray::from_vec(self.storage()?));

            config.setMemoryBalloonDevices(&NSArray::from_vec(vec![Id::into_super(
                VZVirtioTraditionalMemoryBalloonDeviceConfiguration::new(),
            )]));
            config.setEntropyDevices(&NSArray::from_vec(vec![Id::into_super(VZVirtioEntropyDeviceConfiguration::new())]));

            Ok(config)
        }
    }

    fn boot_loader(&self) -> Retained<VZEFIBootLoader> {
        unsafe {
            let url = NSURL::initFileURLWithPath(NSURL::alloc(), &NSString::from_str(&self.dir.nvram_path.to_string_lossy()));
            let store = VZEFIVariableStore::initWithURL(VZEFIVariableStore::alloc(), &url);
            let loader = VZEFIBootLoader::new();
            loader.setVariableStore(Option::Some(&store));
            loader
        }
    }

    fn storage(&self) -> Result<Vec<Retained<VZStorageDeviceConfiguration>>, Exception> {
        let disk = disk(&self.dir.disk_path)?;
        let mut storage = vec![disk];
        if let Option::Some(ref mount_path) = self.mount {
            let disk = mount(mount_path)?;
            storage.push(disk)
        }
        Ok(storage)
    }
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

fn mount(mount: &Path) -> Result<Retained<VZStorageDeviceConfiguration>, Exception> {
    unsafe {
        let attachment = catch(|| {
            let url = NSURL::initFileURLWithPath(NSURL::alloc(), &NSString::from_str(&mount.to_string_lossy()));
            VZDiskImageStorageDeviceAttachment::initWithURL_readOnly_error(VZDiskImageStorageDeviceAttachment::alloc(), &url, true)
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
