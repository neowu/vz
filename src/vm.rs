use std::process;
use std::sync::Arc;
use std::time::Duration;

use block2::StackBlock;
use dispatch::Queue;
use log::error;
use log::info;
use objc2::rc::Retained;
use objc2_foundation::run_on_main;
use objc2_foundation::MainThreadBound;
use objc2_foundation::NSError;
use objc2_virtualization::VZVirtualMachine;

pub mod gui_delegate;
pub mod linux;
pub mod mac_os;
pub mod mac_os_installer;
pub mod vm_delegate;

pub fn start_vm(vm: Arc<MainThreadBound<Retained<VZVirtualMachine>>>) {
    run_on_main(|marker| {
        info!("start vm");
        let vm = vm.get(marker);
        let block = &StackBlock::new(|err: *mut NSError| {
            if err.is_null() {
                info!("vm started");
            } else {
                error!("vm failed to start, error={}", unsafe { (*err).localizedDescription() });
                process::exit(1);
            }
        });
        unsafe {
            vm.startWithCompletionHandler(block);
        }
    });
}

pub fn stop_vm(vm: Arc<MainThreadBound<Retained<VZVirtualMachine>>>) {
    run_on_main(|marker| {
        info!("stop vm");
        if request_stop_vm(vm.get(marker)) {
            Queue::main().exec_after(Duration::from_secs(15), || force_stop_vm(vm));
        } else {
            force_stop_vm(vm);
        }
    });
}

fn request_stop_vm(vm: &Retained<VZVirtualMachine>) -> bool {
    unsafe {
        if vm.canRequestStop() {
            info!("request vm to stop");
            if let Err(err) = vm.requestStopWithError() {
                error!("failed to request vm to stop, error={}", err.localizedDescription());
                process::exit(1);
            }
            return true;
        }
        false
    }
}

fn force_stop_vm(vm: Arc<MainThreadBound<Retained<VZVirtualMachine>>>) {
    run_on_main(|marker| {
        info!("force to stop vm");
        let vm = vm.get(marker);
        if unsafe { vm.canStop() } {
            let block = &StackBlock::new(|err: *mut NSError| {
                if err.is_null() {
                    info!("vm stopped");
                    process::exit(0);
                } else {
                    error!("vm failed to stop, error={}", unsafe { (*err).localizedDescription() });
                    process::exit(1);
                }
            });
            unsafe {
                vm.stopWithCompletionHandler(block);
            }
        } else {
            process::exit(1);
        }
    });
}
