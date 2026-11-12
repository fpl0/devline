import Darwin
import Foundation

public enum BoardError: LocalizedError {
    case invalid(String)
    case changedOnDisk
    public var errorDescription: String? {
        switch self {
        case .invalid(let message): message
        case .changedOnDisk:
            "This workspace changed in another window. Reload it before saving; your draft is still open."
        }
    }
}

public struct Idea: Codable, Equatable, Identifiable, Sendable {
    public let id: UUID
    public fileprivate(set) var title: String
    public fileprivate(set) var notes: String
    public fileprivate(set) var tags: [String]
    public fileprivate(set) var isDone: Bool
    public fileprivate(set) var isPinned: Bool
    public let createdAt: Date
    public fileprivate(set) var updatedAt: Date

    public var markdown: String {
        "# \(title)\n\n\(notes)\n\n---\nTags: \(tags.joined(separator: ", "))\n"
    }
}

public enum IdeaFilter: String, CaseIterable, Identifiable {
    case all = "All"
    case open = "Notes"
    case done = "Archive"
    public var id: Self { self }
}

public struct IdeaBoard: Codable, Equatable, Sendable {
    public private(set) var schemaVersion = 1
    public private(set) var ideas: [Idea] = []
    public init() {}
    public var allTags: [String] { Set(ideas.flatMap(\.tags)).sorted() }

    public static func normalizedTags(_ text: String) -> [String] {
        Array(
            Set(
                text.split(separator: ",").map {
                    $0.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
                }.filter { !$0.isEmpty })
        ).sorted()
    }

    private static func valid(title: String, notes: String, tags: [String]) -> Bool {
        !title.isEmpty && title.count <= 200 && notes.count <= 200_000 && tags.count <= 12
            && title.rangeOfCharacter(from: .controlCharacters) == nil
            && tags.allSatisfy {
                $0.count <= 40 && !$0.isEmpty
                    && $0.rangeOfCharacter(from: .controlCharacters) == nil
            }
    }

    @discardableResult
    public mutating func add(title: String, notes: String, tags: String = "", now: Date = Date())
        -> Idea?
    {
        let title = title.trimmingCharacters(in: .whitespacesAndNewlines)
        let normalized = Self.normalizedTags(tags)
        guard Self.valid(title: title, notes: notes, tags: normalized) else { return nil }
        let idea = Idea(
            id: UUID(), title: title, notes: notes, tags: normalized, isDone: false,
            isPinned: false, createdAt: now, updatedAt: now)
        ideas.insert(idea, at: 0)
        return idea
    }

    @discardableResult
    public mutating func edit(
        _ id: Idea.ID, title: String, notes: String, tags: String? = nil, now: Date = Date()
    ) -> Bool {
        let title = title.trimmingCharacters(in: .whitespacesAndNewlines)
        guard let index = ideas.firstIndex(where: { $0.id == id }) else { return false }
        let normalized = tags.map(Self.normalizedTags) ?? ideas[index].tags
        guard Self.valid(title: title, notes: notes, tags: normalized) else { return false }
        ideas[index].title = title
        ideas[index].notes = notes
        ideas[index].tags = normalized
        ideas[index].updatedAt = max(now, ideas[index].createdAt)
        return true
    }

    public mutating func toggleDone(_ id: Idea.ID) {
        guard let index = ideas.firstIndex(where: { $0.id == id }) else { return }
        ideas[index].isDone.toggle()
        ideas[index].updatedAt = max(Date(), ideas[index].createdAt)
    }
    public mutating func togglePinned(_ id: Idea.ID) {
        guard let index = ideas.firstIndex(where: { $0.id == id }) else { return }
        ideas[index].isPinned.toggle()
    }
    public mutating func remove(_ id: Idea.ID) { ideas.removeAll { $0.id == id } }

    public func visibleIdeas(_ filter: IdeaFilter, matching query: String, tag: String? = nil)
        -> [Idea]
    {
        let terms = query.split(whereSeparator: \.isWhitespace).map(String.init)
        return ideas.filter { idea in
            let status = filter == .all || idea.isDone == (filter == .done)
            let haystack = ([idea.title, idea.notes] + idea.tags).joined(separator: " ")
            return status && (tag == nil || idea.tags.contains(tag!))
                && terms.allSatisfy { haystack.localizedStandardContains($0) }
        }.sorted {
            if $0.isPinned != $1.isPinned { return $0.isPinned }
            if $0.updatedAt != $1.updatedAt { return $0.updatedAt > $1.updatedAt }
            return $0.id.uuidString < $1.id.uuidString
        }
    }

    public func validated() throws -> IdeaBoard {
        guard schemaVersion == 1, Set(ideas.map(\.id)).count == ideas.count,
            ideas.allSatisfy({
                Self.valid(title: $0.title, notes: $0.notes, tags: $0.tags)
                    && $0.tags == Self.normalizedTags($0.tags.joined(separator: ","))
                    && $0.updatedAt >= $0.createdAt
            })
        else { throw BoardError.invalid("This is not a valid version 1 Fieldnotes workspace.") }
        return self
    }

    /// Merge backups without silently replacing a different note with the same identity.
    public mutating func merge(_ other: IdeaBoard) throws -> Int {
        _ = try other.validated()
        let known = Dictionary(uniqueKeysWithValues: ideas.map { ($0.id, $0) })
        for idea in other.ideas {
            if let existing = known[idea.id], existing != idea {
                throw BoardError.invalid(
                    "A different version of '\(idea.title)' already exists. No notes were imported."
                )
            }
        }
        let additions = other.ideas.filter { known[$0.id] == nil }
        ideas.append(contentsOf: additions)
        return additions.count
    }
}

public struct BoardFile {
    public let url: URL
    public init(url: URL) { self.url = url }
    public func load() throws -> IdeaBoard {
        do {
            return try JSONDecoder().decode(IdeaBoard.self, from: Data(contentsOf: url)).validated()
        } catch let error as CocoaError where error.code == .fileReadNoSuchFile {
            return IdeaBoard()
        }
    }
    public func save(_ board: IdeaBoard, expecting previous: IdeaBoard? = nil) throws {
        _ = try board.validated()
        try FileManager.default.createDirectory(
            at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
        let descriptor = open(url.path + ".lock", O_CREAT | O_RDWR, 0o600)
        guard descriptor >= 0 else { throw BoardError.invalid("Cannot open workspace lock.") }
        defer { close(descriptor) }
        guard flock(descriptor, LOCK_EX) == 0 else {
            throw BoardError.invalid("Cannot lock workspace.")
        }
        defer { flock(descriptor, LOCK_UN) }
        if let previous, try load() != previous { throw BoardError.changedOnDisk }
        try Self.encode(board).write(to: url, options: .atomic)
    }
    public static func encode(_ board: IdeaBoard) throws -> Data {
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        return try encoder.encode(board)
    }
}
