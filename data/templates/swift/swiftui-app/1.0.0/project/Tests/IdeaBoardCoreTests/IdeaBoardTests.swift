import Foundation
import IdeaBoardCore
import Testing

private func temporaryPath() -> URL {
    let root =
        ProcessInfo.processInfo.environment["TMPDIR"].map {
            URL(fileURLWithPath: $0, isDirectory: true)
        } ?? FileManager.default.temporaryDirectory
    return root.appending(path: UUID().uuidString)
}

@Test func trimsTitlesAndRejectsBlankIdeas() throws {
    var board = IdeaBoard()
    let blank = board.add(title: " \n", notes: "Draft")
    #expect(blank == nil)
    let added = board.add(title: "  Try SwiftUI  ", notes: "Make a small app")
    let idea = try #require(added)
    #expect(idea.title == "Try SwiftUI")
    #expect(!idea.isDone)
    #expect(board.ideas == [idea])
}

@Test func editsAndFiltersCompletedIdeasByTitleOrNotes() throws {
    var board = IdeaBoard()
    let experiment = board.add(title: "Try a framework", notes: "Build a list")
    let reference = board.add(title: "Read", notes: "SwiftUI App reference")
    let first = try #require(experiment)
    let second = try #require(reference)
    board.toggleDone(first.id)
    board.edit(first.id, title: "SwiftUI experiment", notes: "Build a list")

    #expect(board.visibleIdeas(.open, matching: " swiftui ").map(\.id) == [second.id])
    #expect(board.visibleIdeas(.done, matching: "SWIFTUI").map(\.id) == [first.id])
    #expect(board.visibleIdeas(.all, matching: "missing").isEmpty)

    board.remove(first.id)
    #expect(board.ideas.map(\.id) == [second.id])
}

@Test func savesAndReloadsLocalIdeas() throws {
    let root = temporaryPath()
    defer { try? FileManager.default.removeItem(at: root) }
    let file = BoardFile(url: root.appending(path: "ideas.json"))
    #expect(try file.load() == IdeaBoard())

    var board = IdeaBoard()
    let added = board.add(title: "Work offline", notes: "Keep the docs nearby")
    let idea = try #require(added)
    board.toggleDone(idea.id)
    try file.save(board)
    #expect(try file.load() == board)
}

@Test func readingInvalidDataReportsAnErrorAndPreservesTheFile() throws {
    let url = temporaryPath()
    defer { try? FileManager.default.removeItem(at: url) }
    let data = Data("not JSON".utf8)
    try data.write(to: url)

    #expect(throws: DecodingError.self) { try BoardFile(url: url).load() }
    #expect(try Data(contentsOf: url) == data)
}

@Test func tagsPinsArchiveAndSearchCompose() throws {
    var board = IdeaBoard()
    let oldValue = board.add(
        title: "Network notes", notes: "Study concurrent systems",
        tags: " Swift, networking, SWIFT ", now: Date(timeIntervalSince1970: 10))
    let old = try #require(oldValue)
    let recentValue = board.add(
        title: "Reading", notes: "Another thought", tags: "books",
        now: Date(timeIntervalSince1970: 20))
    let recent = try #require(recentValue)
    #expect(old.tags == ["networking", "swift"])
    board.togglePinned(old.id)
    #expect(board.visibleIdeas(.all, matching: "").first?.id == old.id)
    #expect(
        board.visibleIdeas(.open, matching: "systems swift", tag: "networking").map(\.id) == [
            old.id
        ])
    board.toggleDone(old.id)
    #expect(board.visibleIdeas(.open, matching: "").map(\.id) == [recent.id])
    #expect(board.visibleIdeas(.done, matching: "swift").count == 1)
    #expect(board.edit(old.id, title: "", notes: "bad") == false)
    #expect(board.ideas.first(where: { $0.id == old.id })?.title == "Network notes")
}

@Test func backupMergeIsIdempotentAndRejectsConflictsAtomically() throws {
    var original = IdeaBoard()
    let noteValue = original.add(title: "A note", notes: "Useful text")
    let note = try #require(noteValue)
    var imported = try JSONDecoder().decode(IdeaBoard.self, from: BoardFile.encode(original))
    #expect(try original.merge(imported) == 0)
    imported.add(title: "Another note", notes: "New text")
    imported.edit(note.id, title: "Changed", notes: "Different version")
    let before = original
    #expect(throws: BoardError.self) { try original.merge(imported) }
    #expect(original == before)
    imported.remove(note.id)
    #expect(try original.merge(imported) == 1)
    #expect(original.ideas.count == 2)
}

@Test func staleWriterCannotReplaceNewerWorkspace() throws {
    let root = temporaryPath()
    defer { try? FileManager.default.removeItem(at: root) }
    let file = BoardFile(url: root.appendingPathComponent("workspace.json"))
    let original = IdeaBoard()
    var first = original
    first.add(title: "First writer", notes: "")
    try file.save(first, expecting: original)
    var stale = original
    stale.add(title: "Stale writer", notes: "")
    #expect(throws: BoardError.self) { try file.save(stale, expecting: original) }
    #expect(try file.load() == first)
}

@Test func invalidWorkspaceMetadataIsRejected() throws {
    var board = IdeaBoard()
    board.add(title: "One", notes: "Body")
    let data = try BoardFile.encode(board)
    var object = try #require(JSONSerialization.jsonObject(with: data) as? [String: Any])
    let ideas = try #require(object["ideas"] as? [[String: Any]])
    object["ideas"] = ideas + ideas
    let duplicate = try JSONDecoder().decode(
        IdeaBoard.self, from: JSONSerialization.data(withJSONObject: object))
    #expect(throws: BoardError.self) { try duplicate.validated() }
    #expect(board.add(title: String(repeating: "a", count: 201), notes: "") == nil)
    #expect(board.add(title: "Valid", notes: "", tags: String(repeating: "a", count: 41)) == nil)
}
