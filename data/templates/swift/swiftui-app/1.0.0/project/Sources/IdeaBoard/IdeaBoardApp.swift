import AppKit
import Foundation
import IdeaBoardCore
import SwiftUI
import UniformTypeIdentifiers

@main
struct IdeaBoardApp: App {
    private let file = BoardFile(
        url: (ProcessInfo.processInfo.environment["FIELDNOTES_DATA_DIR"].map {
            URL(fileURLWithPath: $0, isDirectory: true)
        }
            ?? URL.applicationSupportDirectory.appending(
                path: Bundle.main.bundleIdentifier ?? "local.devline.fieldnotes",
                directoryHint: .isDirectory))
            .appending(path: "workspace.json")
    )

    var body: some Scene {
        Window("Fieldnotes", id: "board") {
            BoardView(file: file)
        }
        .defaultSize(width: 850, height: 550)
    }
}

private struct IdeaDraft: Identifiable {
    let id = UUID()
    let original: Idea.ID?
    let originalValue: Idea?
    var title: String
    var notes: String
    var tags: String

    init(_ idea: Idea? = nil) {
        original = idea?.id
        originalValue = idea
        title = idea?.title ?? ""
        notes = idea?.notes ?? ""
        tags = idea?.tags.joined(separator: ", ") ?? ""
    }
}

private struct BoardView: View {
    let file: BoardFile
    @State private var board: IdeaBoard?
    @State private var selection: Idea.ID?
    @State private var filter = IdeaFilter.open
    @State private var query = ""
    @State private var selectedTag: String?
    @State private var notice: String?
    @State private var draft: IdeaDraft?
    @State private var pendingDelete: Idea?
    @State private var failure: String?

    private var selectedIdea: Idea? {
        board?.visibleIdeas(filter, matching: query, tag: selectedTag).first { $0.id == selection }
    }

    var body: some View {
        Group {
            if let board {
                boardContent(board)
            } else {
                ContentUnavailableView {
                    Label("Fieldnotes", systemImage: "note.text")
                } description: {
                    Text("Load your local notes workspace to continue.")
                } actions: {
                    Button("Load Workspace", action: load)
                }
            }
        }
        .safeAreaInset(edge: .bottom) {
            HStack {
                Text(
                    notice
                        ?? "\(board?.ideas.count ?? 0) \(board?.ideas.count == 1 ? "note" : "notes") · Saved on this Mac"
                )
                Spacer()
            }.font(.caption).foregroundStyle(.secondary).padding(8).background(.bar)
        }
        .frame(minWidth: 760, minHeight: 480)
        .task { if board == nil { load() } }
        .sheet(item: $draft) { draft in
            DraftView(draft: draft, failure: $failure, reload: load) { save($0) }
        }
        .confirmationDialog(
            "Permanently delete note?",
            isPresented: Binding(
                get: { pendingDelete != nil }, set: { if !$0 { pendingDelete = nil } }),
            presenting: pendingDelete
        ) { idea in
            Button("Delete Note", role: .destructive) { change { $0.remove(idea.id) } }
        } message: { idea in
            Text(idea.title)
        }
        .alert(
            "Workspace Needs Attention",
            isPresented: Binding(
                get: { failure != nil && draft == nil }, set: { if !$0 { failure = nil } }
            )
        ) {
            Button("OK") { failure = nil }
            Button("Reload Workspace") {
                failure = nil
                load()
            }
        } message: {
            Text(failure ?? "")
        }
    }

    private func boardContent(_ board: IdeaBoard) -> some View {
        NavigationSplitView {
            VStack(spacing: 0) {
                Picker("Filter notes", selection: $filter) {
                    ForEach(IdeaFilter.allCases) { Text($0.rawValue).tag($0) }
                }
                .pickerStyle(.segmented)
                .labelsHidden()
                .padding(12)

                Picker("Tag", selection: $selectedTag) {
                    Text("All tags").tag(Optional<String>.none)
                    ForEach(board.allTags, id: \.self) { Text($0).tag(Optional($0)) }
                }.padding(.horizontal, 12).padding(.bottom, 8)
                List(
                    board.visibleIdeas(filter, matching: query, tag: selectedTag),
                    selection: $selection
                ) { idea in
                    VStack(alignment: .leading, spacing: 5) {
                        Label(
                            idea.title,
                            systemImage: idea.isPinned
                                ? "pin.fill" : (idea.isDone ? "archivebox" : "note.text")
                        )
                        .font(.headline).lineLimit(1)
                        Text(idea.notes.replacingOccurrences(of: "\n", with: " "))
                            .font(.caption).foregroundStyle(.secondary).lineLimit(2)
                        if !idea.tags.isEmpty {
                            Text(idea.tags.map { "#" + $0 }.joined(separator: " "))
                                .font(.caption2).foregroundStyle(.secondary).lineLimit(1)
                        }
                    }.padding(.vertical, 5).tag(idea.id)
                        .contextMenu {
                            Button(idea.isPinned ? "Unpin" : "Pin") {
                                change { $0.togglePinned(idea.id) }
                            }
                            Button(idea.isDone ? "Restore" : "Archive") {
                                change { $0.toggleDone(idea.id) }
                            }
                        }
                }
                .listStyle(.sidebar)
            }
            .navigationSplitViewColumnWidth(min: 205, ideal: 250)
        } detail: {
            if let idea = selectedIdea {
                ideaDetail(idea)
            } else if board.ideas.isEmpty {
                ContentUnavailableView {
                    Label("A place for your thinking", systemImage: "lightbulb")
                } description: {
                    Text("Capture notes, add tags, and pin the ideas you want to return to.")
                } actions: {
                    Button("New Note") { draft = IdeaDraft() }
                }
            } else if board.visibleIdeas(filter, matching: query, tag: selectedTag).isEmpty {
                ContentUnavailableView("No Matching Notes", systemImage: "magnifyingglass")
            } else {
                ContentUnavailableView("Select a Note", systemImage: "lightbulb")
            }
        }
        .searchable(text: $query, placement: .sidebar, prompt: "Search notes and tags")
        .toolbar {
            Button {
                draft = IdeaDraft()
            } label: {
                Label("New Note", systemImage: "plus")
            }
            .keyboardShortcut("n", modifiers: .command)
            .help("Create a note")
            Menu {
                Button("Import Workspace…", action: importWorkspace)
                Button("Export Workspace…", action: exportWorkspace)
                Button("Reload Workspace", action: load)
            } label: {
                Label("Workspace", systemImage: "ellipsis.circle")
            }
        }
    }

    private func ideaDetail(_ idea: Idea) -> some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 18) {
                Text(idea.title).font(.largeTitle.weight(.semibold)).textSelection(.enabled)
                HStack {
                    Button(idea.isDone ? "Restore" : "Archive") {
                        change { $0.toggleDone(idea.id) }
                    }
                    .buttonStyle(.borderedProminent)
                    Button(idea.isPinned ? "Unpin" : "Pin") { change { $0.togglePinned(idea.id) } }
                    Button("Export Markdown…") { exportNote(idea) }
                    Button("Edit") { draft = IdeaDraft(idea) }
                    Button("Delete…", role: .destructive) { pendingDelete = idea }
                }
                HStack {
                    Text(idea.tags.map { "#" + $0 }.joined(separator: "  "))
                    Spacer()
                    Text(idea.updatedAt, format: .dateTime.day().month().year())
                }.font(.caption).foregroundStyle(.secondary)
                Divider()
                Text(idea.notes.isEmpty ? "No notes yet." : idea.notes)
                    .foregroundStyle(idea.notes.isEmpty ? .secondary : .primary)
                    .textSelection(.enabled)
            }
            .padding(28)
            .frame(maxWidth: .infinity, alignment: .leading)
        }
    }

    private func importWorkspace() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = [.json]
        panel.allowsMultipleSelection = false
        guard panel.runModal() == .OK, let url = panel.url, var next = board else { return }
        do {
            let incoming = try JSONDecoder().decode(IdeaBoard.self, from: Data(contentsOf: url))
            let count = try next.merge(incoming)
            guard change({ $0 = next }) else { return }
            notice = "Imported \(count) notes."
        } catch { failure = error.localizedDescription }
    }

    private func exportWorkspace() {
        guard let board else { return }
        let panel = NSSavePanel()
        panel.allowedContentTypes = [.json]
        panel.nameFieldStringValue = "fieldnotes-backup.json"
        guard panel.runModal() == .OK, let url = panel.url else { return }
        do { try BoardFile.encode(board).write(to: url, options: .atomic) } catch {
            failure = error.localizedDescription
        }
    }

    private func exportNote(_ idea: Idea) {
        let panel = NSSavePanel()
        panel.allowedContentTypes = [.plainText]
        panel.nameFieldStringValue = "note.md"
        guard panel.runModal() == .OK, let url = panel.url else { return }
        do { try idea.markdown.write(to: url, atomically: true, encoding: .utf8) } catch {
            failure = error.localizedDescription
        }
    }

    private func load() {
        do {
            board = try file.load()
            failure = nil
        } catch { failure = error.localizedDescription }
    }

    private func save(_ draft: IdeaDraft) -> Bool {
        if let id = draft.original {
            guard board?.ideas.first(where: { $0.id == id }) == draft.originalValue else {
                failure =
                    "This note changed or was removed while you were editing. Your draft is intact. Reload the workspace, then use Save as New to preserve both versions."
                return false
            }
            guard var next = board,
                next.edit(id, title: draft.title, notes: draft.notes, tags: draft.tags)
            else {
                failure =
                    "This note no longer exists, or the draft exceeds the limits. Copy your draft before closing it and create a new note."
                return false
            }
            return change { $0 = next }
        }
        var added: Idea?
        guard change({ added = $0.add(title: draft.title, notes: draft.notes, tags: draft.tags) }),
            let added
        else { return false }
        filter = .all
        query = ""
        selectedTag = nil
        selection = added.id
        return true
    }

    @discardableResult
    private func change(_ update: (inout IdeaBoard) -> Void) -> Bool {
        guard var next = board else { return false }
        update(&next)
        do {
            try file.save(next, expecting: board)
            board = next
            failure = nil
            return true
        } catch {
            failure = error.localizedDescription
            return false
        }
    }
}

private struct DraftView: View {
    @Environment(\.dismiss) private var dismiss
    @FocusState private var titleFocused: Bool
    @State var draft: IdeaDraft
    @Binding var failure: String?
    let reload: () -> Void
    let save: (IdeaDraft) -> Bool

    private var validDraft: Bool {
        var board = IdeaBoard()
        return board.add(title: draft.title, notes: draft.notes, tags: draft.tags) != nil
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text(draft.original == nil ? "New Note" : "Edit Note").font(.title2.weight(.semibold))
            TextField("Title", text: $draft.title)
                .textFieldStyle(.roundedBorder)
                .focused($titleFocused)
            TextField("Tags, separated by commas", text: $draft.tags).textFieldStyle(.roundedBorder)
            Text("Notes").font(.headline)
            TextEditor(text: $draft.notes)
                .font(.body)
                .frame(minHeight: 130)
                .accessibilityLabel("Notes")
                .border(.quaternary)
            if let failure {
                Text(failure).foregroundStyle(.red).font(.callout).textSelection(.enabled)
                Button("Reload Saved Workspace", action: reload)
            }
            HStack {
                Spacer()
                Button("Cancel") {
                    failure = nil
                    dismiss()
                }.keyboardShortcut(.cancelAction)
                if draft.original != nil {
                    Button("Save as New") {
                        var copy = IdeaDraft()
                        copy.title = draft.title
                        copy.notes = draft.notes
                        copy.tags = draft.tags
                        if save(copy) { dismiss() }
                    }.disabled(!validDraft)
                }
                Button("Save") { if save(draft) { dismiss() } }
                    .keyboardShortcut(.defaultAction)
                    .disabled(!validDraft)
            }
            if !validDraft {
                Text(
                    "Use a title up to 200 characters, at most 12 tags of 40 characters each, and notes up to 200,000 characters."
                ).font(.caption).foregroundStyle(.secondary)
            }
        }
        .padding(24)
        .frame(width: 440)
        .onAppear { titleFocused = true }
    }
}
