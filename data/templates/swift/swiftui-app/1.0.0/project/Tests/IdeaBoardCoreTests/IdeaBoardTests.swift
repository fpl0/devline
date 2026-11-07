import Foundation
import IdeaBoardCore
import Testing

private func temporaryPath() -> URL {
    let root = ProcessInfo.processInfo.environment["TMPDIR"].map {
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
