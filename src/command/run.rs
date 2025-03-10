use std::env::current_exe;
use std::fs::File;
use std::path::PathBuf;
use std::process;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use std::thread;

use clap::Args;
use clap::ValueHint;
use dispatch2::MainThreadBound;
use dispatch2::ffi::dispatch_main;
use log::info;
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
use objc2_foundation::MainThreadMarker;
use objc2_foundation::NSPoint;
use objc2_foundation::NSRect;
use objc2_foundation::NSSize;
use objc2_foundation::NSString;
use objc2_foundation::ns_string;
use objc2_virtualization::VZVirtualMachine;
use objc2_virtualization::VZVirtualMachineDelegate;
use objc2_virtualization::VZVirtualMachineView;
use signal_hook::consts::signal::SIGINT;
use signal_hook::consts::signal::SIGQUIT;
use signal_hook::consts::signal::SIGTERM;
use signal_hook::iterator::Signals;

use crate::config::vm_config::Os;
use crate::config::vm_dir;
use crate::util::path::PathExtension;
use crate::vm;
use crate::vm::gui_delegate::GuiDelegate;
use crate::vm::linux;
use crate::vm::mac_os;
use crate::vm::vm_delegate::VmDelegate;

#[derive(Args)]
pub struct Run {
    #[arg(help = "vm name")]
    name: String,
    #[arg(long, help = "open UI window", default_value_t = false)]
    gui: bool,
    #[arg(short, help = "run vm in background", default_value_t = false)]
    detached: bool,
    #[arg(long, help = "attach disk image in read only mode, e.g. --mount=debian.iso", value_hint = ValueHint::FilePath)]
    mount: Option<PathBuf>,
}

impl Run {
    pub fn execute(&self) {
        self.validate();

        let name = &self.name;
        let dir = vm_dir::vm_dir(name);
        if !dir.initialized() {
            panic!("vm not initialized, name={name}");
        }

        if self.detached {
            return run_in_background(name);
        }

        let config = dir.load_config();

        // must after vm_dir.load_config(), it cloese config file and release all fd
        // must hold lock reference, otherwise fd will be deallocated, and release all locks
        let _lock = dir.lock();

        let marker = MainThreadMarker::new().unwrap();
        let vm = match config.os {
            Os::Linux => linux::create_vm(&dir, &config, self.gui, self.mount.as_ref()),
            Os::MacOs => mac_os::create_vm(&dir, &config, marker),
        };
        let proto: Retained<ProtocolObject<dyn VZVirtualMachineDelegate>> = ProtocolObject::from_retained(VmDelegate::new());
        unsafe {
            vm.setDelegate(Some(&proto));
        }
        let vm = Arc::new(MainThreadBound::new(vm, marker));
        vm::start_vm(name, Arc::clone(&vm));

        handle_signal(name.to_string(), Arc::clone(&vm));

        if self.gui {
            let auto_reconfig_display = matches!(&config.os, Os::MacOs);
            run_gui(name, marker, vm, auto_reconfig_display);
        } else {
            unsafe {
                dispatch_main();
            }
        }
    }

    fn validate(&self) {
        if let Some(path) = &self.mount {
            if !path.exists() {
                panic!("mount does not exist, path={}", path.to_string_lossy());
            }
        }

        if self.detached && (self.gui || self.mount.is_some()) {
            panic!("-d must not be used with --gui and --mount");
        }
    }
}

#[allow(clippy::zombie_processes)]
fn run_in_background(name: &str) {
    let log_path = PathBuf::from("~/Library/Logs/vz.log").to_absolute_path();

    if let Ok(metadata) = log_path.metadata() {
        if !metadata.is_file() || metadata.permissions().readonly() {
            panic!("log file is not writable, path={}", log_path.to_string_lossy());
        }
    }

    let mut command = Command::new(current_exe().unwrap_or_else(|err| panic!("failed to get current command path, err={err}")));
    command.args(["run", name]);
    command.stdout(log_file_io(&log_path));
    command.stderr(log_file_io(&log_path));
    command.spawn().unwrap_or_else(|err| panic!("failed to run command, err={err}"));
    info!("vm launched in background, check log in {}", log_path.to_string_lossy());
}

fn log_file_io(log_path: &PathBuf) -> Stdio {
    Stdio::from(
        File::options()
            .create(true)
            .append(true)
            .open(log_path)
            .unwrap_or_else(|err| panic!("failed to open log file, err={err}")),
    )
}

fn handle_signal(name: String, vm: Arc<MainThreadBound<Retained<VZVirtualMachine>>>) {
    let mut signals = Signals::new([SIGTERM, SIGINT, SIGQUIT]).unwrap();
    thread::spawn(move || {
        let signal = signals.forever().next().unwrap();
        info!("recived signal, signal={signal}, name={name}, pid={}", process::id());
        match signal {
            SIGTERM | SIGINT | SIGQUIT => {
                vm::stop_vm(name, vm);
            }
            _ => {
                info!("signal ignored");
            }
        }
    });
}

fn run_gui(name: &str, marker: MainThreadMarker, vm: Arc<MainThreadBound<Retained<VZVirtualMachine>>>, auto_reconfig_display: bool) {
    let app = NSApplication::sharedApplication(marker);
    app.setActivationPolicy(NSApplicationActivationPolicy::Regular);

    let window = unsafe {
        NSWindow::initWithContentRect_styleMask_backing_defer_screen(
            marker.alloc(),
            NSRect {
                origin: NSPoint::new(0.0, 0.0),
                size: NSSize::new(1024.0, 768.0),
            },
            NSWindowStyleMask::Titled | NSWindowStyleMask::Resizable | NSWindowStyleMask::Closable,
            NSBackingStoreType::Buffered,
            false,
            Option::None,
        )
    };
    window.setTitle(&NSString::from_str(name));

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
        machine_view.setAutomaticallyReconfiguresDisplay(auto_reconfig_display);
        machine_view.setVirtualMachine(Some(vm.get(marker)));
        machine_view.setAutoresizingMask(NSAutoresizingMaskOptions::ViewWidthSizable | NSAutoresizingMaskOptions::ViewHeightSizable);
        window.contentView().unwrap().addSubview(&machine_view);
    }

    let proto: Retained<ProtocolObject<dyn NSWindowDelegate>> = ProtocolObject::from_retained(GuiDelegate::new(marker, vm, name));
    window.setDelegate(Some(&proto));

    window.makeKeyAndOrderFront(Option::None);
    app.run();
}
