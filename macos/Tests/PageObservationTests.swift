import Combine
import Foundation

@MainActor private final class TestModel: ObservableObject {
    @Published var libraryRevision = 0
    @Published var completedJobs: [String] = []
}

@main struct PageObservationTests {
    @MainActor static func main() async {
        let model = TestModel()
        let gate = PageVisibilityGate(isVisible: true)
        let parent = PageObservation<TestModel>()
        let nested = PageObservation<TestModel>()
        parent.bind(model, gate: gate)
        nested.bind(model, gate: gate)
        var parentEvents = 0
        var nestedEvents = 0
        var lastObservedRevision = 0
        let parentToken = parent.objectWillChange.sink {
            parentEvents += 1
            lastObservedRevision = model.libraryRevision
        }
        let nestedToken = nested.objectWillChange.sink { nestedEvents += 1 }
        model.libraryRevision = 1
        precondition(parentEvents == 1 && nestedEvents == 1, "visible source updates must reach both views")

        gate.setVisible(false)
        for revision in 2...20 { model.libraryRevision = revision }
        model.completedJobs = ["download-completed"]
        precondition(parentEvents == 1 && nestedEvents == 1, "hidden updates must not invalidate parent or nested view")
        gate.setVisible(true)
        precondition(parentEvents == 1, "catch-up must not publish inside visibility update")
        await drainMainQueue()
        precondition(parentEvents == 2 && nestedEvents == 2, "many hidden changes must coalesce into one catch-up per view")
        precondition(lastObservedRevision == 20 && model.completedJobs == ["download-completed"], "catch-up observes current library revision and jobs")

        gate.setVisible(false)
        gate.setVisible(true)
        await drainMainQueue()
        precondition(parentEvents == 2 && nestedEvents == 2, "clean reveal must not invalidate either view")

        gate.setVisible(false)
        model.libraryRevision = 21
        gate.setVisible(true)
        gate.setVisible(false)
        await drainMainQueue()
        precondition(parentEvents == 2, "hide before queued reveal must suppress stale catch-up")
        gate.setVisible(true)
        await drainMainQueue()
        precondition(parentEvents == 3 && nestedEvents == 3, "suppressed catch-up must remain pending for next reveal")

        let replacement = TestModel()
        parent.bind(replacement, gate: gate)
        model.libraryRevision = 22
        precondition(parentEvents == 3 && nestedEvents == 4, "rebinding must detach the previous source")
        replacement.libraryRevision = 1
        precondition(parentEvents == 4, "replacement source must be observed")

        let outsidePage = PageObservation<TestModel>()
        outsidePage.bind(replacement, gate: nil)
        var outsideEvents = 0
        let outsideToken = outsidePage.objectWillChange.sink { outsideEvents += 1 }
        replacement.libraryRevision = 2
        precondition(outsideEvents == 1, "views outside a kept page must continue observing normally")

        let wrapper = PageObservedObject(wrappedValue: replacement)
        wrapper.projectedValue.libraryRevision.wrappedValue = 3
        precondition(replacement.libraryRevision == 3, "projected bindings must write to the shared source")
        precondition(wrapper.projectedValue.libraryRevision.wrappedValue == 3, "projected bindings must read current values")
        withExtendedLifetime([parentToken, nestedToken, outsideToken]) {}
        print("Page observation tests passed")
    }

    @MainActor private static func drainMainQueue() async {
        await withCheckedContinuation { continuation in
            DispatchQueue.main.async { continuation.resume() }
        }
    }
}
