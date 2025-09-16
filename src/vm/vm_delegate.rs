use std::process;

use objc2::AllocAnyThread;
use objc2::define_class;
use objc2::msg_send;
use objc2::rc::Retained;
use objc2_foundation::NSError;
use objc2_foundation::NSObject;
use objc2_foundation::NSObjectProtocol;
use objc2_virtualization::VZNetworkDevice;
use objc2_virtualization::VZVirtualMachine;
use objc2_virtualization::VZVirtualMachineDelegate;
use tracing::error;
use tracing::info;

define_class!(
    #[unsafe(super = NSObject)]
    #[name = "VmDelegate"]
    pub struct VmDelegate;

    unsafe impl NSObjectProtocol for VmDelegate {}

    unsafe impl VZVirtualMachineDelegate for VmDelegate {
        #[unsafe(method(guestDidStopVirtualMachine:))]
        fn guest_did_stop_virtual_machine(&self, _: &VZVirtualMachine) {
            info!("guest has stopped the vm");
            process::exit(0);
        }

        #[unsafe(method(virtualMachine:didStopWithError:))]
        fn virtual_machine_did_stop_with_error(&self, _: &VZVirtualMachine, err: &NSError) {
            error!("guest has stopped the vm due to error, err={}", err.localizedDescription());
            process::exit(1);
        }

        #[unsafe(method(virtualMachine:networkDevice:attachmentWasDisconnectedWithError:))]
        fn virtual_machine_network_device_attachment_was_disconnected_with_error(
            &self,
            _: &VZVirtualMachine,
            network_device: &VZNetworkDevice,
            err: &NSError,
        ) {
            error!("vm network disconnected, device={network_device:?}, err={}", err.localizedDescription());
            process::exit(1);
        }
    }
);

impl VmDelegate {
    pub fn new() -> Retained<Self> {
        unsafe { msg_send![VmDelegate::alloc(), init] }
    }
}
