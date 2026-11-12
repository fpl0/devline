import Darwin
import Foundation

public struct FocusFile {
    public let url: URL
    public init(url: URL) { self.url = url }
    public func load() throws -> SessionJournal {
        do {
            return try JSONDecoder().decode(SessionJournal.self, from: Data(contentsOf: url))
                .validated()
        } catch let error as CocoaError where error.code == .fileReadNoSuchFile {
            return SessionJournal()
        }
    }
    public func save(_ state: SessionJournal, expecting previous: SessionJournal) throws {
        _ = try state.validated()
        try FileManager.default.createDirectory(
            at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
        let descriptor = open(url.path + ".lock", O_CREAT | O_RDWR, 0o600)
        guard descriptor >= 0 else { throw FocusError.invalid("Cannot open session lock.") }
        defer { close(descriptor) }
        guard flock(descriptor, LOCK_EX) == 0 else {
            throw FocusError.invalid("Cannot lock sessions.")
        }
        defer { flock(descriptor, LOCK_UN) }
        guard try load() == previous else {
            throw FocusError.invalid(
                "Sessions changed in another app instance. Reload before making changes.")
        }
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        try encoder.encode(state).write(to: url, options: .atomic)
    }
}
