import Darwin
import Foundation

public struct LedgerFile {
    public let url: URL
    public init(url: URL) { self.url = url }
    /// The advisory lock covers read, validation, mutation, and atomic replacement.
    public func transaction<T>(_ operation: (inout Ledger) throws -> T) throws -> T {
        try FileManager.default.createDirectory(
            at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
        let descriptor = open(url.path + ".lock", O_CREAT | O_RDWR, 0o600)
        guard descriptor >= 0 else {
            throw LedgerError.invalid(
                "Cannot open ledger lock: \(String(cString: strerror(errno))).")
        }
        defer { close(descriptor) }
        guard flock(descriptor, LOCK_EX) == 0 else {
            throw LedgerError.invalid("Cannot lock the ledger.")
        }
        defer { flock(descriptor, LOCK_UN) }
        let original: Ledger
        do {
            original = try JSONDecoder().decode(Ledger.self, from: Data(contentsOf: url))
                .validated()
        } catch let error as CocoaError where error.code == .fileReadNoSuchFile {
            original = try Ledger()
        }
        var next = original
        let result = try operation(&next)
        if next != original { try encoded(next).write(to: url, options: .atomic) }
        return result
    }
    public func encoded(_ ledger: Ledger) throws -> Data {
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        return try encoder.encode(ledger)
    }
}
