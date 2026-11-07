import Foundation

public struct Idea: Codable, Equatable, Identifiable {
    public let id: UUID
    public fileprivate(set) var title: String
    public fileprivate(set) var notes: String
    public fileprivate(set) var isDone: Bool
}

public enum IdeaFilter: String, CaseIterable, Identifiable {
    case all = "All"
    case open = "Open"
    case done = "Done"

    public var id: Self { self }
}

public struct IdeaBoard: Codable, Equatable {
    public private(set) var ideas: [Idea] = []

    public init() {}

    @discardableResult
    public mutating func add(title: String, notes: String) -> Idea? {
        let title = title.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !title.isEmpty else { return nil }
        let idea = Idea(id: UUID(), title: title, notes: notes, isDone: false)
        ideas.insert(idea, at: 0)
        return idea
    }

    public mutating func edit(_ id: Idea.ID, title: String, notes: String) {
        let title = title.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !title.isEmpty, let index = ideas.firstIndex(where: { $0.id == id }) else { return }
        ideas[index].title = title
        ideas[index].notes = notes
    }

    public mutating func toggleDone(_ id: Idea.ID) {
        guard let index = ideas.firstIndex(where: { $0.id == id }) else { return }
        ideas[index].isDone.toggle()
    }

    public mutating func remove(_ id: Idea.ID) {
        ideas.removeAll { $0.id == id }
    }

    public func visibleIdeas(_ filter: IdeaFilter, matching query: String) -> [Idea] {
        let term = query.trimmingCharacters(in: .whitespacesAndNewlines)
        return ideas.filter { idea in
            let statusMatches = filter == .all || idea.isDone == (filter == .done)
            let textMatches = term.isEmpty || idea.title.localizedStandardContains(term)
                || idea.notes.localizedStandardContains(term)
            return statusMatches && textMatches
        }
    }
}

public struct BoardFile {
    public let url: URL

    public init(url: URL) { self.url = url }

    public func load() throws -> IdeaBoard {
        do {
            return try JSONDecoder().decode(IdeaBoard.self, from: Data(contentsOf: url))
        } catch let error as CocoaError where error.code == .fileReadNoSuchFile {
            return IdeaBoard()
        }
    }

    public func save(_ board: IdeaBoard) throws {
        try FileManager.default.createDirectory(at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        try encoder.encode(board).write(to: url, options: .atomic)
    }
}
