import Foundation
import IdeaBoardCore
import SwiftUI

@main
struct IdeaBoardApp: App {
    private let file = BoardFile(
        url: URL.applicationSupportDirectory
            .appending(path: Bundle.main.bundleIdentifier ?? "local.devline.idea-board", directoryHint: .isDirectory)
            .appending(path: "ideas.json")
    )

    var body: some Scene {
        Window("Idea Board", id: "board") {
            BoardView(file: file)
        }
        .defaultSize(width: 850, height: 550)
    }
}

private struct IdeaDraft: Identifiable {
    let id = UUID()
    let original: Idea.ID?
    var title: String
    var notes: String

    init(_ idea: Idea? = nil) {
        original = idea?.id
        title = idea?.title ?? ""
        notes = idea?.notes ?? ""
    }
}

private struct BoardView: View {
    let file: BoardFile
    @State private var board: IdeaBoard?
    @State private var selection: Idea.ID?
    @State private var filter = IdeaFilter.all
    @State private var query = ""
    @State private var draft: IdeaDraft?
    @State private var pendingDelete: Idea?
    @State private var failure: String?

    private var selectedIdea: Idea? {
        board?.visibleIdeas(filter, matching: query).first { $0.id == selection }
    }

    var body: some View {
        Group {
            if let board {
                boardContent(board)
            } else {
                ContentUnavailableView {
                    Label("Your Ideas", systemImage: "lightbulb")
                } description: {
                    Text("Load the idea board to continue.")
                } actions: {
                    Button("Load Board", action: load)
                }
            }
        }
        .frame(minWidth: 620, minHeight: 400)
        .task { if board == nil { load() } }
        .sheet(item: $draft) { draft in
            DraftView(draft: draft) { save($0) }
        }
        .confirmationDialog(
            "Delete idea?",
            isPresented: Binding(get: { pendingDelete != nil }, set: { if !$0 { pendingDelete = nil } }),
            presenting: pendingDelete
        ) { idea in
            Button("Delete Idea", role: .destructive) { change { $0.remove(idea.id) } }
        } message: { idea in
            Text(idea.title)
        }
        .alert("Unable to Open or Save Board", isPresented: Binding(
            get: { failure != nil }, set: { if !$0 { failure = nil } }
        )) {
            Button("OK") { failure = nil }
        } message: {
            Text(failure ?? "")
        }
    }

    private func boardContent(_ board: IdeaBoard) -> some View {
        NavigationSplitView {
            VStack(spacing: 0) {
                Picker("Filter ideas", selection: $filter) {
                    ForEach(IdeaFilter.allCases) { Text($0.rawValue).tag($0) }
                }
                .pickerStyle(.segmented)
                .labelsHidden()
                .padding(12)

                List(board.visibleIdeas(filter, matching: query), selection: $selection) { idea in
                    Label(idea.title, systemImage: idea.isDone ? "checkmark.circle.fill" : "circle")
                        .tag(idea.id)
                }
                .listStyle(.sidebar)
            }
            .navigationSplitViewColumnWidth(min: 205, ideal: 250)
        } detail: {
            if let idea = selectedIdea {
                ideaDetail(idea)
            } else if board.ideas.isEmpty {
                ContentUnavailableView {
                    Label("Make room for an idea", systemImage: "lightbulb")
                } description: {
                    Text("Keep experiments and small projects here as you think of them.")
                } actions: {
                    Button("New Idea") { draft = IdeaDraft() }
                }
            } else if board.visibleIdeas(filter, matching: query).isEmpty {
                ContentUnavailableView("No Matching Ideas", systemImage: "magnifyingglass")
            } else {
                ContentUnavailableView("Select an Idea", systemImage: "lightbulb")
            }
        }
        .searchable(text: $query, placement: .sidebar, prompt: "Search titles and notes")
        .toolbar {
            Button {
                draft = IdeaDraft()
            } label: {
                Label("New Idea", systemImage: "plus")
            }
            .keyboardShortcut("n", modifiers: .command)
            .help("Create an idea")
        }
    }

    private func ideaDetail(_ idea: Idea) -> some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 18) {
                Text(idea.title).font(.largeTitle.weight(.semibold)).textSelection(.enabled)
                HStack {
                    Button(idea.isDone ? "Reopen" : "Mark Done") { change { $0.toggleDone(idea.id) } }
                        .buttonStyle(.borderedProminent)
                    Button("Edit") { draft = IdeaDraft(idea) }
                    Button("Delete…", role: .destructive) { pendingDelete = idea }
                }
                Divider()
                Text(idea.notes.isEmpty ? "No notes yet." : idea.notes)
                    .foregroundStyle(idea.notes.isEmpty ? .secondary : .primary)
                    .textSelection(.enabled)
            }
            .padding(28)
            .frame(maxWidth: .infinity, alignment: .leading)
        }
    }

    private func load() {
        do { board = try file.load() }
        catch { failure = error.localizedDescription }
    }

    private func save(_ draft: IdeaDraft) -> Bool {
        if let id = draft.original {
            return change { $0.edit(id, title: draft.title, notes: draft.notes) }
        }
        var added: Idea?
        guard change({ added = $0.add(title: draft.title, notes: draft.notes) }), let added else { return false }
        filter = .all
        query = ""
        selection = added.id
        return true
    }

    @discardableResult
    private func change(_ update: (inout IdeaBoard) -> Void) -> Bool {
        guard var next = board else { return false }
        update(&next)
        do {
            try file.save(next)
            board = next
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
    let save: (IdeaDraft) -> Bool

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text(draft.original == nil ? "New Idea" : "Edit Idea").font(.title2.weight(.semibold))
            TextField("Title", text: $draft.title)
                .textFieldStyle(.roundedBorder)
                .focused($titleFocused)
            Text("Notes").font(.headline)
            TextEditor(text: $draft.notes)
                .font(.body)
                .frame(minHeight: 130)
                .accessibilityLabel("Notes")
                .border(.quaternary)
            HStack {
                Spacer()
                Button("Cancel") { dismiss() }.keyboardShortcut(.cancelAction)
                Button("Save") { if save(draft) { dismiss() } }
                    .keyboardShortcut(.defaultAction)
                    .disabled(draft.title.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
            }
        }
        .padding(24)
        .frame(width: 440)
        .onAppear { titleFocused = true }
    }
}
