use std::process;

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
use tracing::error;
use tracing::info;

declare_class!(
    pub struct VMDelegate;

    unsafe impl ClassType for VMDelegate {
        type Super = NSObject;
        type Mutability = mutability::InteriorMutable;
        const NAME: &'static str = "VMDelegate";
    }

    impl DeclaredClass for VMDelegate {
    }

    unsafe impl NSObjectProtocol for VMDelegate {}

    unsafe impl VZVirtualMachineDelegate for VMDelegate {
        #[method(guestDidStopVirtualMachine:)]
        fn guest_did_stop_virtual_machine(&self, _: &VZVirtualMachine) {
            info!("guest has stopped the vm");
            process::exit(0);
        }

        #[method(virtualMachine:didStopWithError:)]
        fn virtual_machine_did_stop_with_error(&self, _: &VZVirtualMachine, error: &NSError) {
            error!("guest has stopped the vm due to error, error={error}");
            process::exit(1);
        }

        #[method(virtualMachine:networkDevice:attachmentWasDisconnectedWithError:)]
        fn virtual_machine_network_device_attachment_was_disconnected_with_error(&self, _: &VZVirtualMachine, network_device: &VZNetworkDevice, error: &NSError) {
            error!("vm network disconnected, device={:?}, error={error}", network_device);
            process::exit(1);
        }
    }
);

impl VMDelegate {
    pub fn new() -> Retained<Self> {
        unsafe { msg_send_id![Self::alloc(), init] }
    }
}
