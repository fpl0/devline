import FocusCore
import Foundation
import Testing

private let start = Date(timeIntervalSince1970: 100_000)

@Test func pauseAndResumeExcludePausedTime() throws {
    var state = SessionJournal()
    try state.configure(focusMinutes: 1, breakMinutes: 1)
    try state.start(label: "Read a paper", at: start)
    try state.pause(at: start.addingTimeInterval(20))
    #expect(state.active?.remaining(at: start.addingTimeInterval(900)) == 40)
    try state.resume(at: start.addingTimeInterval(120))
    #expect(state.advance(to: start.addingTimeInterval(159)) == false)
    #expect(state.advance(to: start.addingTimeInterval(160)) == true)
    #expect(state.history.first?.focusedSeconds == 60)
    #expect(state.history.first?.finishedAt == start.addingTimeInterval(160))
    #expect(state.advance(to: start.addingTimeInterval(500)) == false)
    #expect(state.history.count == 1)
}

@Test func sleepAndReloadFinishAtDeadlineExactlyOnce() throws {
    var state = SessionJournal()
    try state.start(label: "Write", at: start)
    let data = try JSONEncoder().encode(state)
    var recovered = try JSONDecoder().decode(SessionJournal.self, from: data).validated()
    #expect(recovered.advance(to: start.addingTimeInterval(10_000)) == true)
    #expect(recovered.history[0].finishedAt == start.addingTimeInterval(25 * 60))
    #expect(recovered.history[0].completed)
    #expect(recovered.advance(to: start.addingTimeInterval(20_000)) == false)
}

@Test func stoppedFocusIsRecordedAndBreaksDoNotCount() throws {
    var state = SessionJournal()
    try state.start(label: "Research", at: start)
    try state.pause(at: start.addingTimeInterval(31))
    state.stop(at: start.addingTimeInterval(100))
    #expect(state.history[0].focusedSeconds == 31)
    #expect(!state.history[0].completed)
    try state.start(label: "", kind: .rest, at: start.addingTimeInterval(200))
    #expect(state.advance(to: start.addingTimeInterval(600)) == true)
    #expect(state.history.count == 1)
    #expect(state.totalSeconds(on: start) == 31)
}

@Test func invalidTransitionsAndPreferencesLeaveStateIntact() throws {
    var state = SessionJournal()
    #expect(throws: FocusError.self) { try state.configure(focusMinutes: 0, breakMinutes: 5) }
    #expect(throws: FocusError.self) { try state.start(label: " ", at: start) }
    #expect(throws: FocusError.self) { try state.resume(at: start) }
    try state.start(label: "One task", at: start)
    let before = state
    #expect(throws: FocusError.self) { try state.start(label: "Second task", at: start) }
    #expect(state == before)
    #expect(state.active?.remaining(at: start.addingTimeInterval(-100)) == 1500)
}

@Test func pauseAtDeadlineCompletesAndCsvEscapesLabels() throws {
    var state = SessionJournal()
    try state.configure(focusMinutes: 1, breakMinutes: 1)
    try state.start(label: "=Read, \"notes\"", at: start)
    try state.pause(at: start.addingTimeInterval(60))
    #expect(state.active == nil)
    #expect(state.history.count == 1)
    #expect(state.csv.contains("\"'=Read, \"\"notes\"\"\""))
}

@Test func persistenceRejectsStaleWritersAndInvalidFiles() throws {
    let root =
        (ProcessInfo.processInfo.environment["TMPDIR"].map {
            URL(fileURLWithPath: $0, isDirectory: true)
        } ?? FileManager.default.temporaryDirectory).appendingPathComponent(UUID().uuidString)
    defer { try? FileManager.default.removeItem(at: root) }
    let file = FocusFile(url: root.appendingPathComponent("sessions.json"))
    let original = try file.load()
    var state = original
    try state.start(label: "Test", at: start)
    try file.save(state, expecting: original)
    #expect(try file.load() == state)
    #expect(throws: FocusError.self) { try file.save(original, expecting: original) }
    try Data("broken".utf8).write(to: file.url)
    #expect(throws: DecodingError.self) { try file.load() }
    #expect(try Data(contentsOf: file.url) == Data("broken".utf8))
}
