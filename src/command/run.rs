use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use std::time::Duration;

use block2::StackBlock;
use clap::Args;
use dispatch::ffi::dispatch_main;
use futures::stream::StreamExt;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_app_kit::NSApplication;
use objc2_app_kit::NSApplicationActivationPolicy;
use objc2_foundation::run_on_main;
use objc2_foundation::MainThreadBound;
use objc2_foundation::MainThreadMarker;
use objc2_foundation::NSError;
use objc2_virtualization::VZVirtualMachine;
use objc2_virtualization::VZVirtualMachineDelegate;
use signal_hook::consts::SIGINT;
use signal_hook::consts::SIGTERM;
use signal_hook_tokio::Signals;
use tokio::sync::mpsc::error::SendError;
use tokio::time::sleep;
use tracing::error;
use tracing::info;

use crate::config::vm_dir;
use crate::util::exception::Exception;
use crate::vm::delegate::VMDelegate;
use crate::vm::linux::Linux;

#[derive(Args)]
#[command(about = "run vm")]
pub struct Run {
    #[arg(long, help = "vm name")]
    name: String,
    #[arg(long, help = "open UI window", default_value_t = false)]
    gui: bool,
    #[arg(long, help = "attach disk image in read only mode, e.g. --mount=\"debian.iso\"", value_hint = clap::ValueHint::FilePath)]
    mount: Option<PathBuf>,
}

impl Run {
    pub async fn execute(&self) -> Result<(), Exception> {
        let name = &self.name;
        let vm_dir = vm_dir::vm_dir(name);
        if !vm_dir.initialized() {
            return Result::Err(Exception::new(format!("vm not initialized, name={name}")));
        }

        let vm_config = vm_dir.load_config()?;
        let linux = Linux::new(vm_dir, vm_config, self.gui, self.mount.clone());

        let vm = linux.create_vm()?;
        let delegate = VMDelegate::new();
        let proto: &ProtocolObject<dyn VZVirtualMachineDelegate> = ProtocolObject::from_ref(&*delegate);
        unsafe {
            vm.setDelegate(Option::Some(proto));
        }
        let vm = Arc::new(MainThreadBound::new(vm, MainThreadMarker::new().unwrap()));

        let signals = Signals::new([SIGTERM, SIGINT])?;
        let handle = signals.handle();
        tokio::spawn(handle_signals(signals, Arc::clone(&vm)));

        start_vm(Arc::clone(&vm));

        unsafe {
            if self.gui {
                let app = NSApplication::sharedApplication(MainThreadMarker::new().unwrap());
                app.setActivationPolicy(NSApplicationActivationPolicy::Regular);
                app.run();
            } else {
                dispatch_main();
            }
        };

        handle.close();
        Ok(())
    }
}

fn start_vm(vm: Arc<MainThreadBound<Retained<VZVirtualMachine>>>) {
    run_on_main(move |marker| {
        info!("start vm");
        let vm = vm.get(marker);
        unsafe {
            vm.startWithCompletionHandler(&StackBlock::new(|err: *mut NSError| {
                if err.is_null() {
                    info!("vm started");
                } else {
                    error!("vm failed to start, error={}", (*err).localizedDescription());
                }
            }));
        }
    });
}

async fn handle_signals(mut signals: Signals, vm: Arc<MainThreadBound<Retained<VZVirtualMachine>>>) {
    while let Some(signal) = signals.next().await {
        match signal {
            SIGTERM | SIGINT => {
                stop_vm(&vm);
                sleep(Duration::from_secs(15)).await;
                force_stop_vm(&vm);
            }
            _ => unreachable!(),
        }
    }
}

fn stop_vm(holder: &Arc<MainThreadBound<Retained<VZVirtualMachine>>>) {
    run_on_main(|marker| {
        info!("stop vm");
        let vm = holder.get(marker);
        unsafe {
            if vm.canRequestStop() {
                info!("request vm to stop");
                if let Err(err) = vm.requestStopWithError() {
                    error!("failed to request vm to stop, error={}", err.localizedDescription());
                    process::exit(1);
                }
            } else {
                force_stop_vm(holder);
            }
        }
    });
}

fn force_stop_vm(vm: &Arc<MainThreadBound<Retained<VZVirtualMachine>>>) {
    run_on_main(|marker| {
        info!("force to stop vm");
        let vm = vm.get(marker);
        unsafe {
            if vm.canStop() {
                vm.stopWithCompletionHandler(&StackBlock::new(|err: *mut NSError| {
                    if err.is_null() {
                        info!("vm stopped");
                        process::exit(0);
                    } else {
                        error!("vm failed to stop, error={}", (*err).localizedDescription());
                        process::exit(1);
                    }
                }));
            } else {
                process::exit(1);
            }
        }
    });
}

impl<T> From<SendError<T>> for Exception {
    fn from(err: SendError<T>) -> Self {
        Exception::new(err.to_string())
    }
}
