use core::ffi::c_void;
use core::ptr;
use std::path::Path;
use std::process;

use block2::StackBlock;
use dispatch2::MainThreadBound;
use dispatch2::dispatch_main;
use dispatch2::run_on_main;
use objc2::AllocAnyThread;
use objc2::DeclaredClass;
use objc2::define_class;
use objc2::msg_send;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_foundation::MainThreadMarker;
use objc2_foundation::NSDictionary;
use objc2_foundation::NSError;
use objc2_foundation::NSKeyValueChangeKey;
use objc2_foundation::NSKeyValueObservingOptions;
use objc2_foundation::NSNumber;
use objc2_foundation::NSObject;
use objc2_foundation::NSObjectNSKeyValueObserverRegistration;
use objc2_foundation::NSObjectProtocol;
use objc2_foundation::NSProgress;
use objc2_foundation::NSString;
use objc2_foundation::ns_string;
use objc2_virtualization::VZMacOSInstaller;
use objc2_virtualization::VZVirtualMachine;
use tracing::error;
use tracing::info;

use crate::util::path::PathExtension;

pub fn install(vm: Retained<VZVirtualMachine>, ipsw: &Path, marker: MainThreadMarker) {
    let installer = unsafe { VZMacOSInstaller::initWithVirtualMachine_restoreImageURL(VZMacOSInstaller::alloc(), &vm, &ipsw.to_ns_url()) };
    let _observer = VZMacOSInstallerObserver::new(unsafe { installer.progress() });
    let installer = MainThreadBound::new(installer, marker);

    run_on_main(|marker| {
        let installer = installer.get(marker);
        let block = &StackBlock::new(move |err: *mut NSError| {
            if !err.is_null() {
                error!("failed to install macOS, err={}", unsafe { (*err).localizedDescription() });
                process::exit(1);
            } else {
                info!("instal macOS done");
                process::exit(0);
            }
        });
        unsafe {
            installer.installWithCompletionHandler(block);
        }
    });
    dispatch_main();
}

struct Ivars {
    progress: Retained<NSProgress>,
}

define_class!(
    #[unsafe(super = NSObject)]
    #[name = "VZMacOSInstallerObserver"]
    #[ivars = Ivars]
    struct VZMacOSInstallerObserver;

    unsafe impl NSObjectProtocol for VZMacOSInstallerObserver {}

    impl VZMacOSInstallerObserver {
        #[unsafe(method(observeValueForKeyPath:ofObject:change:context:))]
        fn observe_value(
            &self,
            _key_path: Option<&NSString>,
            _object: Option<&AnyObject>,
            change: Option<&NSDictionary<NSKeyValueChangeKey, AnyObject>>,
            _context: *mut c_void,
        ) {
            if let Some(change) = change {
                let new_value = change.objectForKey(ns_string!("new")).unwrap();
                let percent: Retained<NSNumber> = unsafe { Retained::cast_unchecked(new_value) };
                info!("instal progress: {:.2}%", percent.floatValue() * 100.0);
            }
        }
    }
);

impl VZMacOSInstallerObserver {
    fn new(progress: Retained<NSProgress>) -> Retained<Self> {
        let observer = VZMacOSInstallerObserver::alloc().set_ivars(Ivars { progress });
        let observer: Retained<Self> = unsafe { msg_send![super(observer), init] };
        let progress = &observer.ivars().progress;
        unsafe {
            progress.addObserver_forKeyPath_options_context(
                &observer,
                ns_string!("fractionCompleted"),
                NSKeyValueObservingOptions::Initial | NSKeyValueObservingOptions::New,
                ptr::null_mut(),
            );
        }
        observer
    }
}

impl Drop for VZMacOSInstallerObserver {
    fn drop(&mut self) {
        unsafe {
            self.ivars().progress.removeObserver_forKeyPath(self, ns_string!("fractionCompleted"));
        }
    }
}
