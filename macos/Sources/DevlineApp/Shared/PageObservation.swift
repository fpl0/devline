import Combine
import SwiftUI

/// A stable environment reference. Changing visibility does not invalidate all
/// descendants; only observers that missed source updates are notified on reveal.
@MainActor
final class PageVisibilityGate {
    private(set) var isVisible: Bool
    fileprivate let becameVisible = PassthroughSubject<Void, Never>()
    private var revision = 0

    init(isVisible: Bool) { self.isVisible = isVisible }

    func setVisible(_ visible: Bool) {
        guard visible != isVisible else { return }
        isVisible = visible
        revision += 1
        guard visible else { return }
        let scheduled = revision
        // Visibility is set by onChange during a SwiftUI update. Send catch-up
        // notifications on the next main-queue turn, outside that update.
        DispatchQueue.main.async { [weak self] in
            guard let self, self.isVisible, self.revision == scheduled else { return }
            self.becameVisible.send()
        }
    }
}

private struct PageVisibilityGateKey: EnvironmentKey {
    static let defaultValue: PageVisibilityGate? = nil
}

extension EnvironmentValues {
    var pageVisibilityGate: PageVisibilityGate? {
        get { self[PageVisibilityGateKey.self] }
        set { self[PageVisibilityGateKey.self] = newValue }
    }
}

/// Sources used here publish on the main actor, as WorkspaceModel does.
@MainActor
final class PageObservation<Object: ObservableObject>: ObservableObject {
    private var sourceID: ObjectIdentifier?
    private var sourceSubscription: AnyCancellable?
    private weak var gate: PageVisibilityGate?
    private var visibilitySubscription: AnyCancellable?
    private var pending = false

    func bind(_ object: Object, gate: PageVisibilityGate?) {
        if self.gate !== gate {
            self.gate = gate
            visibilitySubscription = gate?.becameVisible.sink { [weak self] in
                self?.catchUp()
            }
        }
        let identity = ObjectIdentifier(object)
        if identity != sourceID {
            sourceID = identity
            pending = false
            sourceSubscription = object.objectWillChange.sink { [weak self] _ in
                self?.sourceWillChange()
            }
        }
    }

    private func sourceWillChange() {
        if gate?.isVisible == false {
            pending = true
        } else {
            pending = false
            objectWillChange.send()
        }
    }

    private func catchUp() {
        guard pending, gate?.isVisible != false else { return }
        pending = false
        objectWillChange.send()
    }
}

/// Observes a shared model only while its kept page is visible. The wrapped
/// object is always the live object, so delayed callbacks read its latest state.
@MainActor
@propertyWrapper
struct PageObservedObject<Object: ObservableObject>: DynamicProperty {
    @Environment(\.pageVisibilityGate) private var gate
    @StateObject private var observation = PageObservation<Object>()
    private let object: Object

    init(wrappedValue: Object) { object = wrappedValue }
    var wrappedValue: Object { object }

    nonisolated mutating func update() {
        MainActor.assumeIsolated { observation.bind(object, gate: gate) }
    }

    var projectedValue: ObservedObject<Object>.Wrapper {
        ObservedObject(wrappedValue: object).projectedValue
    }
}
