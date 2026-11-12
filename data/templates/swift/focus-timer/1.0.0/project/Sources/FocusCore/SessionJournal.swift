import Foundation

public enum FocusError: LocalizedError {
    case invalid(String)
    public var errorDescription: String? {
        switch self {
        case .invalid(let message): message
        }
    }
}

public enum SessionKind: String, Codable, Sendable { case focus, rest }

public struct ActiveSession: Codable, Equatable, Identifiable, Sendable {
    public let id: UUID
    public let label: String
    public let kind: SessionKind
    public let duration: TimeInterval
    public let startedAt: Date
    public fileprivate(set) var deadline: Date?
    public fileprivate(set) var pausedRemaining: TimeInterval
    public var isPaused: Bool { deadline == nil }
    public func remaining(at now: Date) -> TimeInterval {
        max(0, min(duration, deadline.map { $0.timeIntervalSince(now) } ?? pausedRemaining))
    }
}

public struct SessionRecord: Codable, Equatable, Identifiable, Sendable {
    public let id: UUID
    public let label: String
    public let startedAt: Date
    public let finishedAt: Date
    public let focusedSeconds: Int
    public let completed: Bool
}

/// Time is an explicit input so sleep, pause, resume, and recovery have deterministic semantics.
public struct SessionJournal: Codable, Equatable, Sendable {
    public private(set) var schemaVersion = 1
    public private(set) var focusMinutes = 25
    public private(set) var breakMinutes = 5
    public private(set) var active: ActiveSession?
    public private(set) var history: [SessionRecord] = []
    public init() {}

    public mutating func configure(focusMinutes: Int, breakMinutes: Int) throws {
        guard (1...120).contains(focusMinutes), (1...30).contains(breakMinutes) else {
            throw FocusError.invalid("Focus must be 1–120 minutes; breaks must be 1–30 minutes.")
        }
        self.focusMinutes = focusMinutes
        self.breakMinutes = breakMinutes
    }

    public mutating func start(label: String, kind: SessionKind = .focus, at now: Date) throws {
        guard active == nil else {
            throw FocusError.invalid("Finish or stop the current session first.")
        }
        let clean = label.trimmingCharacters(in: .whitespacesAndNewlines)
        guard
            kind == .rest
                || (!clean.isEmpty && clean.count <= 120
                    && clean.rangeOfCharacter(from: .controlCharacters) == nil)
        else {
            throw FocusError.invalid("Name your focus with 1–120 characters and no line breaks.")
        }
        let seconds = TimeInterval((kind == .focus ? focusMinutes : breakMinutes) * 60)
        active = ActiveSession(
            id: UUID(), label: kind == .rest ? "Break" : clean, kind: kind,
            duration: seconds, startedAt: now,
            deadline: now.addingTimeInterval(seconds), pausedRemaining: seconds)
    }

    @discardableResult
    public mutating func advance(to now: Date) -> Bool {
        guard let current = active, let end = current.deadline, end <= now else { return false }
        if current.kind == .focus {
            history.insert(
                SessionRecord(
                    id: current.id, label: current.label, startedAt: current.startedAt,
                    finishedAt: end, focusedSeconds: Int(current.duration), completed: true), at: 0)
        }
        active = nil
        return true
    }

    public mutating func pause(at now: Date) throws {
        if advance(to: now) { return }
        guard var current = active, !current.isPaused else {
            throw FocusError.invalid("No running session to pause.")
        }
        current.pausedRemaining = current.remaining(at: now)
        current.deadline = nil
        active = current
    }

    public mutating func resume(at now: Date) throws {
        guard var current = active, current.isPaused else {
            throw FocusError.invalid("No paused session to resume.")
        }
        current.deadline = now.addingTimeInterval(current.pausedRemaining)
        active = current
    }

    public mutating func stop(at now: Date) {
        if advance(to: now) { return }
        guard let current = active else { return }
        let seconds = Int(current.duration - current.remaining(at: now))
        if current.kind == .focus, seconds > 0 {
            history.insert(
                SessionRecord(
                    id: current.id, label: current.label, startedAt: current.startedAt,
                    finishedAt: max(now, current.startedAt), focusedSeconds: seconds,
                    completed: false), at: 0)
        }
        active = nil
    }

    public func totalSeconds(on date: Date, calendar: Calendar = .current) -> Int {
        history.filter { calendar.isDate($0.finishedAt, inSameDayAs: date) }.reduce(0) {
            $0 + $1.focusedSeconds
        }
    }

    public var csv: String {
        func quote(_ value: String) -> String {
            let safe = value.first.map { "=+-@".contains($0) } == true ? "'" + value : value
            return "\"" + safe.replacingOccurrences(of: "\"", with: "\"\"") + "\""
        }
        let formatter = ISO8601DateFormatter()
        return
            (["id,label,started_at,finished_at,focused_seconds,completed"]
            + history.map {
                [
                    $0.id.uuidString, quote($0.label), formatter.string(from: $0.startedAt),
                    formatter.string(from: $0.finishedAt), String($0.focusedSeconds),
                    String($0.completed),
                ].joined(separator: ",")
            }).joined(separator: "\n") + "\n"
    }

    public func validated() throws -> SessionJournal {
        guard schemaVersion == 1, (1...120).contains(focusMinutes), (1...30).contains(breakMinutes),
            Set(history.map(\.id)).count == history.count,
            history.allSatisfy({
                (1...7200).contains($0.focusedSeconds) && $0.finishedAt >= $0.startedAt
                    && !$0.label.isEmpty && $0.label.count <= 120
            })
        else {
            throw FocusError.invalid("Invalid Focus Sessions data; your file has not been changed.")
        }
        if let active {
            guard active.duration.isFinite, (60...7200).contains(active.duration),
                active.pausedRemaining.isFinite,
                (0...active.duration).contains(active.pausedRemaining),
                active.deadline == nil || active.deadline! >= active.startedAt,
                !active.label.isEmpty, active.label.count <= 120,
                !history.contains(where: { $0.id == active.id })
            else {
                throw FocusError.invalid("Invalid active session; your file has not been changed.")
            }
        }
        return self
    }
}
