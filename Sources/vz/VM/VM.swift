import Foundation
import Virtualization

class VM: NSObject, VZVirtualMachineDelegate, NSWindowDelegate {
    let machine: VZVirtualMachine

    init(_ machine: VZVirtualMachine) {
        self.machine = machine
        super.init()
        machine.delegate = self
    }

    @MainActor
    func start() {
        Logger.info("start vm")

        machine.start(completionHandler: { result in
            switch result {
            case .success:
                Logger.info("vm started")
                return
            case .failure(let error):
                Logger.error("vm failed to start, error=\(error)")
                exit(EXIT_FAILURE)
            }
        })
    }

    @MainActor
    func stop() async throws {
        Logger.info("stop vm")

        if machine.canRequestStop {
            Logger.info("request vm to stop")
            try machine.requestStop()
            try await Task.sleep(nanoseconds: 15_000_000_000)
        }

        Logger.info("force to stop vm")
        if machine.canStop {
            machine.stop { error in
                if let error = error {
                    Logger.info("failed to stop the vm, error=\(error)")
                    exit(EXIT_FAILURE)
                } else {
                    Logger.info("vm stopped")
                    exit(EXIT_SUCCESS)
                }
            }
        } else {
            exit(EXIT_FAILURE)
        }
    }

    func guestDidStop(_ machine: VZVirtualMachine) {
        Logger.info("guest has stopped the vm")
        exit(EXIT_SUCCESS)
    }

    func virtualMachine(_ machine: VZVirtualMachine, didStopWithError error: Error) {
        Logger.error("guest has stopped the vm due to error, error=\(error)")
        exit(EXIT_FAILURE)
    }

    func virtualMachine(_ machine: VZVirtualMachine, networkDevice: VZNetworkDevice, attachmentWasDisconnectedWithError error: Error) {
        Logger.error("vm network disconnected, device=\(networkDevice), error=\(error)")
        exit(EXIT_FAILURE)
    }

    func windowWillClose(_ notification: Notification) {
        Task {
            try await stop()
        }
    }
}
