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
use objc2_virtualization::VZVirtualMachine;

use crate::vm::vm_delegate;

pub struct Ivars {
    vm: Arc<MainThreadBound<Retained<VZVirtualMachine>>>,
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
             vm_delegate::stop_vm(Arc::clone(&self.ivars().vm));
        }
    }
);

impl GuiDelegate {
    pub fn new(marker: MainThreadMarker, vm: Arc<MainThreadBound<Retained<VZVirtualMachine>>>) -> Retained<Self> {
        let this = marker.alloc();
        let this = this.set_ivars(Ivars { vm });
        unsafe { msg_send_id![super(this), init] }
    }
}
