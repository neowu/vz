use std::process;

use log::error;
use log::info;
use objc2::declare_class;
use objc2::msg_send_id;
use objc2::mutability;
use objc2::rc::Retained;
use objc2::ClassType;
use objc2::DeclaredClass;
use objc2_foundation::NSError;
use objc2_foundation::NSObject;
use objc2_foundation::NSObjectProtocol;
use objc2_virtualization::VZNetworkDevice;
use objc2_virtualization::VZVirtualMachine;
use objc2_virtualization::VZVirtualMachineDelegate;

declare_class!(
    pub struct VmDelegate;

    unsafe impl ClassType for VmDelegate {
        type Super = NSObject;
        type Mutability = mutability::Immutable;
        const NAME: &'static str = "VmDelegate";
    }

    impl DeclaredClass for VmDelegate {
    }

    unsafe impl NSObjectProtocol for VmDelegate {}

    unsafe impl VZVirtualMachineDelegate for VmDelegate {
        #[method(guestDidStopVirtualMachine:)]
        fn guest_did_stop_virtual_machine(&self, _: &VZVirtualMachine) {
            info!("guest has stopped the vm");
            process::exit(0);
        }

        #[method(virtualMachine:didStopWithError:)]
        fn virtual_machine_did_stop_with_error(&self, _: &VZVirtualMachine, err: &NSError) {
            error!("guest has stopped the vm due to error, error={}", err.localizedDescription());
            process::exit(1);
        }

        #[method(virtualMachine:networkDevice:attachmentWasDisconnectedWithError:)]
        fn virtual_machine_network_device_attachment_was_disconnected_with_error(&self, _: &VZVirtualMachine, network_device: &VZNetworkDevice, err: &NSError) {
            error!("vm network disconnected, device={network_device:?}, error={}", err.localizedDescription());
            process::exit(1);
        }
    }
);

impl VmDelegate {
    pub fn new() -> Retained<Self> {
        unsafe { msg_send_id![Self::alloc(), init] }
    }
}
