use std::env::current_exe;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;

use clap::Args;
use clap::ValueHint;
use dispatch::ffi::dispatch_main;
use futures::stream::StreamExt;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::sel;
use objc2_app_kit::NSApplication;
use objc2_app_kit::NSApplicationActivationPolicy;
use objc2_app_kit::NSAutoresizingMaskOptions;
use objc2_app_kit::NSBackingStoreType;
use objc2_app_kit::NSMenu;
use objc2_app_kit::NSMenuItem;
use objc2_app_kit::NSWindow;
use objc2_app_kit::NSWindowDelegate;
use objc2_app_kit::NSWindowStyleMask;
use objc2_foundation::ns_string;
use objc2_foundation::CGPoint;
use objc2_foundation::CGSize;
use objc2_foundation::MainThreadBound;
use objc2_foundation::MainThreadMarker;
use objc2_foundation::NSRect;
use objc2_foundation::NSString;
use objc2_virtualization::VZVirtualMachine;
use objc2_virtualization::VZVirtualMachineDelegate;
use objc2_virtualization::VZVirtualMachineView;
use signal_hook::consts::SIGINT;
use signal_hook::consts::SIGQUIT;
use signal_hook::consts::SIGTERM;
use signal_hook_tokio::Signals;
use tokio::time::sleep;
use tracing::info;

use crate::config::vm_config::Os;
use crate::config::vm_dir;
use crate::util::exception::Exception;
use crate::util::path::PathExtension;
use crate::vm::delegate;
use crate::vm::delegate::VMDelegate;
use crate::vm::linux;
use crate::vm::mac_os;

#[derive(Args)]
pub struct Run {
    #[arg(help = "vm name")]
    name: String,
    #[arg(long, help = "open UI window", default_value_t = false)]
    gui: bool,
    #[arg(short, help = "run vm in background", default_value_t = false)]
    detached: bool,
    #[arg(long, help = "attach disk image in read only mode, e.g. --mount=\"debian.iso\"", value_hint = ValueHint::FilePath)]
    mount: Option<PathBuf>,
}

impl Run {
    pub async fn execute(&self) -> Result<(), Exception> {
        let name = &self.name;
        let dir = vm_dir::vm_dir(name);
        if !dir.initialized() {
            return Err(Exception::ValidationError(format!("vm not initialized, name={name}")));
        }
        if self.detached {
            if self.gui || self.mount.is_some() {
                return Err(Exception::ValidationError("-d must not be used with --gui and --mount".to_string()));
            }
            let log_path = &PathBuf::from("~/Library/Logs/vz.log").to_absolute_path();
            if let Ok(metadata) = log_path.metadata() {
                if !metadata.is_file() || metadata.permissions().readonly() {
                    return Err(Exception::ValidationError(format!(
                        "log file is not writable, path={}",
                        log_path.to_string_lossy()
                    )));
                }
            }
            return run_in_background(name, log_path);
        }

        let config = dir.load_config()?;

        // must after vm_dir.load_config(), it cloese config file and release all fd
        // must hold lock reference, otherwise fd will be deallocated, and release all locks
        let _lock = dir.lock()?;

        let vm = match config.os {
            Os::Linux => linux::create_vm(&dir, &config, self.gui, self.mount.as_ref())?,
            Os::MacOs => mac_os::create_vm(&dir, &config)?,
        };

        let marker = MainThreadMarker::new().unwrap();
        let delegate = VMDelegate::new(marker, MainThreadBound::new(vm.clone(), marker));
        let proto: &ProtocolObject<dyn VZVirtualMachineDelegate> = ProtocolObject::from_ref(&*delegate);
        unsafe {
            vm.setDelegate(Some(proto));
        }

        let signals = Signals::new([SIGTERM, SIGINT])?;
        let handle = signals.handle();

        let bound = MainThreadBound::new(vm.clone(), marker);
        let task = tokio::spawn(async move { handle_signals(signals, &bound).await });

        delegate::start_vm(&MainThreadBound::new(vm.clone(), marker));

        if self.gui {
            let automatically_reconfigures_display = matches!(&config.os, Os::MacOs);
            run_gui(name, vm, delegate, automatically_reconfigures_display);
        } else {
            unsafe {
                dispatch_main();
            }
        }

        handle.close();
        task.await?;
        Ok(())
    }
}

fn run_in_background(name: &str, log_path: &Path) -> Result<(), Exception> {
    let mut command = Command::new(current_exe()?);
    command.args(["run", name]);
    command.stdout(Stdio::from(File::create(log_path)?));
    command.stderr(Stdio::from(File::create(log_path)?));
    command.spawn()?;
    info!("vm launched in background, check log in {}", log_path.to_string_lossy());
    Ok(())
}

fn run_gui(name: &str, vm: Retained<VZVirtualMachine>, delegate: Retained<VMDelegate>, automatically_reconfigures_display: bool) {
    let marker = MainThreadMarker::new().unwrap();

    let app = NSApplication::sharedApplication(marker);
    app.setActivationPolicy(NSApplicationActivationPolicy::Regular);

    let window = unsafe {
        NSWindow::initWithContentRect_styleMask_backing_defer_screen(
            marker.alloc(),
            NSRect {
                origin: CGPoint::new(0.0, 0.0),
                size: CGSize::new(1024.0, 768.0),
            },
            NSWindowStyleMask::Titled | NSWindowStyleMask::Resizable | NSWindowStyleMask::Closable,
            NSBackingStoreType::NSBackingStoreBuffered,
            false,
            Option::None,
        )
    };
    window.setTitle(&NSString::from_str(name));
    let proto: &ProtocolObject<dyn NSWindowDelegate> = ProtocolObject::from_ref(&*delegate);
    window.setDelegate(Some(proto));

    let menu = NSMenu::new(marker);
    let menu_item = NSMenuItem::new(marker);
    let sub_menu = NSMenu::new(marker);
    unsafe { sub_menu.addItemWithTitle_action_keyEquivalent(&NSString::from_str(&format!("Stop {name}...")), Some(sel!(close)), ns_string!("q")) };
    menu_item.setSubmenu(Some(&sub_menu));
    menu.addItem(&menu_item);
    app.setMainMenu(Some(&menu));
    unsafe {
        let machine_view = VZVirtualMachineView::initWithFrame(marker.alloc(), window.contentLayoutRect());
        machine_view.setCapturesSystemKeys(true);
        machine_view.setAutomaticallyReconfiguresDisplay(automatically_reconfigures_display);
        machine_view.setVirtualMachine(Some(&vm));
        machine_view.setAutoresizingMask(NSAutoresizingMaskOptions::NSViewWidthSizable | NSAutoresizingMaskOptions::NSViewHeightSizable);

        window.contentView().unwrap().addSubview(&machine_view);
    }
    window.makeKeyAndOrderFront(Option::None);

    unsafe { app.run() };
}

async fn handle_signals(mut signals: Signals, bound: &MainThreadBound<Retained<VZVirtualMachine>>) {
    while let Some(signal) = signals.next().await {
        match signal {
            SIGTERM | SIGINT | SIGQUIT => {
                delegate::stop_vm(bound);
                sleep(Duration::from_secs(15)).await;
                delegate::force_stop_vm(bound);
            }
            _ => unreachable!(),
        }
    }
}
