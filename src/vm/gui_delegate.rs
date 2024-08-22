use std::sync::Arc;

use objc2::declare_class;
use objc2::msg_send_id;
use objc2::mutability;
use objc2::rc::Retained;
use objc2::ClassType;
use objc2::DeclaredClass;
use objc2_app_kit::NSWindowDelegate;
use objc2_foundation::MainThreadBound;
use objc2_foundation::MainThreadMarker;
use objc2_foundation::NSNotification;
use objc2_foundation::NSObject;
use objc2_foundation::NSObjectProtocol;
use objc2_foundation::NSString;
use objc2_virtualization::VZVirtualMachine;

use crate::vm;

pub struct Ivars {
    vm: Arc<MainThreadBound<Retained<VZVirtualMachine>>>,
    name: Retained<NSString>,
}

declare_class!(
    pub struct GuiDelegate;

    unsafe impl ClassType for GuiDelegate {
        type Super = NSObject;
        type Mutability = mutability::MainThreadOnly;
        const NAME: &'static str = "GuiDelegate";
    }

    impl DeclaredClass for GuiDelegate {
        type Ivars = Ivars;
    }

    unsafe impl NSObjectProtocol for GuiDelegate {}

    unsafe impl NSWindowDelegate for GuiDelegate {
        #[method(windowWillClose:)]
        fn window_will_close(&self, _: &NSNotification) {
            let ivars = self.ivars();
             vm::stop_vm(ivars.name.to_string(), Arc::clone(&ivars.vm));
        }
    }
);

impl GuiDelegate {
    pub fn new(marker: MainThreadMarker, vm: Arc<MainThreadBound<Retained<VZVirtualMachine>>>, name: &str) -> Retained<Self> {
        let this = marker.alloc();
        let this = this.set_ivars(Ivars {
            vm,
            name: NSString::from_str(name),
        });
        unsafe { msg_send_id![super(this), init] }
    }
}
