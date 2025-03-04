use std::path::Path;

use log::info;
use objc2::AllocAnyThread;
use objc2::rc::Retained;
use objc2_app_kit::NSScreen;
use objc2_foundation::MainThreadMarker;
use objc2_foundation::NSArray;
use objc2_foundation::NSData;
use objc2_foundation::NSDataBase64DecodingOptions;
use objc2_foundation::NSSize;
use objc2_foundation::NSString;
use objc2_virtualization::VZAudioDeviceConfiguration;
use objc2_virtualization::VZDiskImageCachingMode;
use objc2_virtualization::VZDiskImageStorageDeviceAttachment;
use objc2_virtualization::VZDiskImageSynchronizationMode;
use objc2_virtualization::VZGraphicsDeviceConfiguration;
use objc2_virtualization::VZHostAudioOutputStreamSink;
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
use objc2_virtualization::VZVirtioSoundDeviceConfiguration;
use objc2_virtualization::VZVirtioSoundDeviceOutputStreamConfiguration;
use objc2_virtualization::VZVirtioTraditionalMemoryBalloonDeviceConfiguration;
use objc2_virtualization::VZVirtualMachine;
use objc2_virtualization::VZVirtualMachineConfiguration;

use crate::config::vm_config::VmConfig;
use crate::config::vm_dir::VmDir;
use crate::util::path::PathExtension;

pub fn create_vm(dir: &VmDir, config: &VmConfig, marker: MainThreadMarker) -> Retained<VZVirtualMachine> {
    info!("create macOS vm, name={}", dir.name());
    let vz_config = create_vm_config(dir, config, marker);
    unsafe {
        vz_config
            .validateWithError()
            .unwrap_or_else(|err| panic!("virtual machine config validation error, err={}", err.localizedDescription()));
        VZVirtualMachine::initWithConfiguration(VZVirtualMachine::alloc(), &vz_config)
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

fn create_vm_config(dir: &VmDir, config: &VmConfig, marker: MainThreadMarker) -> Retained<VZVirtualMachineConfiguration> {
    unsafe {
        let vz_config = VZVirtualMachineConfiguration::new();
        vz_config.setCPUCount(config.cpu);
        vz_config.setMemorySize(config.memory);

        vz_config.setBootLoader(Some(&VZMacOSBootLoader::new()));
        vz_config.setPlatform(&platform(dir, config));

        vz_config.setGraphicsDevices(&NSArray::from_retained_slice(&[display(1920, 1080, marker)]));
        vz_config.setAudioDevices(&NSArray::from_retained_slice(&[audio()]));
        vz_config.setKeyboards(&NSArray::from_retained_slice(&[Retained::into_super(VZMacKeyboardConfiguration::new())]));
        vz_config.setPointingDevices(&NSArray::from_retained_slice(&[Retained::into_super(VZMacTrackpadConfiguration::new())]));

        vz_config.setNetworkDevices(&NSArray::from_retained_slice(&[config.network()]));
        vz_config.setStorageDevices(&NSArray::from_retained_slice(&[disk(&dir.disk_path)]));

        vz_config.setMemoryBalloonDevices(&NSArray::from_retained_slice(&[Retained::into_super(
            VZVirtioTraditionalMemoryBalloonDeviceConfiguration::new(),
        )]));
        vz_config.setEntropyDevices(&NSArray::from_retained_slice(&[Retained::into_super(
            VZVirtioEntropyDeviceConfiguration::new(),
        )]));

        if let Some(sharing) = config.sharing_directories() {
            vz_config.setDirectorySharingDevices(&NSArray::from_retained_slice(&[sharing]));
        }
        vz_config
    }
}

fn audio() -> Retained<VZAudioDeviceConfiguration> {
    unsafe {
        let stream = VZVirtioSoundDeviceOutputStreamConfiguration::new();
        stream.setSink(Some(&Retained::into_super(VZHostAudioOutputStreamSink::new())));
        let audio = VZVirtioSoundDeviceConfiguration::new();
        audio.setStreams(&NSArray::from_retained_slice(&[Retained::into_super(stream)]));
        Retained::into_super(audio)
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
        Retained::into_super(platform)
    }
}

fn disk(disk: &Path) -> Retained<VZStorageDeviceConfiguration> {
    unsafe {
        let attachment = VZDiskImageStorageDeviceAttachment::initWithURL_readOnly_cachingMode_synchronizationMode_error(
            VZDiskImageStorageDeviceAttachment::alloc(),
            &disk.to_ns_url(),
            false,
            VZDiskImageCachingMode::Automatic,
            VZDiskImageSynchronizationMode::Fsync,
        )
        .unwrap_or_else(|err| panic!("failed to create disk, err={}", err.localizedDescription()));
        let disk = VZVirtioBlockDeviceConfiguration::initWithAttachment(VZVirtioBlockDeviceConfiguration::alloc(), &attachment);
        Retained::into_super(disk)
    }
}

fn display(width: isize, height: isize, marker: MainThreadMarker) -> Retained<VZGraphicsDeviceConfiguration> {
    let screen = NSScreen::mainScreen(marker).unwrap();
    unsafe {
        let display = VZMacGraphicsDeviceConfiguration::new();
        display.setDisplays(&NSArray::from_retained_slice(&[
            VZMacGraphicsDisplayConfiguration::initForScreen_sizeInPoints(
                VZMacGraphicsDisplayConfiguration::alloc(),
                &screen,
                NSSize::new(width as f64, height as f64),
            ),
        ]));
        Retained::into_super(display)
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
