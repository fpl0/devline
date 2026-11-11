import Network
import SwiftUI

/// Whether this Mac currently has a usable network path. Devline works either way; this only
/// decides whether to offer the steps that download something.
@MainActor
final class Connectivity: ObservableObject {
    @Published private(set) var online = true
    private let monitor = NWPathMonitor()

    init() {
        monitor.pathUpdateHandler = { [weak self] path in
            let reachable = path.status == .satisfied
            Task { @MainActor in
                if self?.online != reachable { self?.online = reachable }
            }
        }
        monitor.start(queue: DispatchQueue(label: "devline.connectivity"))
    }

    deinit { monitor.cancel() }
}
