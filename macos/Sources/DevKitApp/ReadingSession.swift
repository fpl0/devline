import AppKit
import SwiftUI

struct ReadingReference: Decodable, Identifiable, Sendable {
    let id, title, packId, stack, version, sourceUrl: String
    let visitedAt: Double?
    let savedAt: Double?
    let anchor, focusText: String?
    let installed: Bool
}

struct ReadingLibrary: Decodable, Sendable {
    var saved: [ReadingReference] = []
    var recent: [ReadingReference] = []
}

struct ResolvedReference: Decodable, Sendable {
    let id: String
    let anchor, focusText: String?
}

struct ReferenceRequest: Equatable {
    let token = UUID()
    let location: ReferenceLocation
    var restoring = false
}

@MainActor
final class ReadingSession: ObservableObject {
    @Published private(set) var library = ReadingLibrary()
    @Published private(set) var history = ReferenceHistory()
    @Published var request: ReferenceRequest?
    @Published var error: String?
    private var operation: Task<Void, Never>?
    private var operationID = 0

    func refresh() async { await enqueue { CoreRequest("reading.list") } }

    func open(_ location: ReferenceLocation) { request = ReferenceRequest(location: location) }

    func move(_ direction: Int) {
        guard let location = history.move(direction) else { return }
        request = ReferenceRequest(location: location, restoring: true)
    }

    func didOpen(_ location: ReferenceLocation, restoring: Bool, searchPreview: Bool) async {
        if !restoring { history.record(location, searchPreview: searchPreview) }
        await enqueue { CoreRequest("reading.visit", id: location.id, anchor: location.anchor, focusText: location.focusText) }
    }

    func updatePosition(id: String, fraction: Double) { history.updatePosition(id: id, fraction: fraction) }
    func isSaved(_ id: String) -> Bool { library.saved.contains { $0.id == id } }

    func toggleSaved(_ location: ReferenceLocation) async {
        await enqueue {
            CoreRequest(self.isSaved(location.id) ? "reading.remove" : "reading.save", id: location.id,
                        anchor: location.anchor, focusText: location.focusText)
        }
    }

    private func enqueue(_ makeRequest: @escaping @MainActor () -> CoreRequest) async {
        let previous = operation
        operationID += 1
        let id = operationID
        let task = Task { @MainActor in
            await previous?.value
            do { self.library = try await Core.request(makeRequest()) }
            catch is CancellationError {}
            catch { self.error = error.localizedDescription }
        }
        operation = task
        await task.value
        if operationID == id { operation = nil }
    }
}

func displayDocVersion(_ version: String) -> String {
    let components = version.components(separatedBy: "+devline.")
    guard components.count == 2 else { return version }
    return "\(components[0]) · revision \(components[1])"
}

struct ReferenceShelf: View {
    @ObservedObject var reading: ReadingSession
    let open: (ReadingReference) -> Void
    var initialSaved = true
    @Environment(\.dismiss) private var dismiss
    @State private var saved = true
    @State private var filter = ""
    private var references: [ReadingReference] {
        (saved ? reading.library.saved : reading.library.recent).filter {
            filter.isEmpty || "\($0.title) \($0.packId) \($0.version)".localizedCaseInsensitiveContains(filter)
        }
    }
    var body: some View {
        VStack(spacing: 14) {
            HStack {
                Text("Your References").font(.title2.weight(.semibold))
                Spacer()
                Button("Done") { dismiss() }.keyboardShortcut(.cancelAction)
            }
            Picker("References", selection: $saved) {
                Text("Saved").tag(true)
                Text("Recent").tag(false)
            }.pickerStyle(.segmented)
            TextField("Filter references", text: $filter).textFieldStyle(.roundedBorder)
            if references.isEmpty {
                ContentUnavailableView(saved ? "No Saved References" : "No Recent References",
                                       systemImage: saved ? "bookmark" : "clock",
                                       description: Text(saved ? "Save any article with ⌘D to keep it here." : "Articles you open appear here, ready to revisit."))
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                List(references) { reference in
                    HStack(spacing: 12) {
                        Button {
                            dismiss()
                            open(reference)
                        } label: {
                            VStack(alignment: .leading, spacing: 5) {
                                Text(ReaderMarkup.title(reference.title, fallback: reference.packId))
                                    .font(.body.weight(.medium)).lineLimit(2)
                                Text("\(reference.packId) · \(displayDocVersion(reference.version))")
                                    .font(.caption).foregroundStyle(.secondary)
                                if !reference.installed {
                                    Text("This version is no longer installed").font(.caption).foregroundStyle(.secondary)
                                }
                            }.frame(maxWidth: .infinity, alignment: .leading).contentShape(Rectangle())
                        }.buttonStyle(.plain).disabled(!reference.installed)
                        if saved {
                            Button {
                                Task { await reading.toggleSaved(ReferenceLocation(id: reference.id)) }
                            } label: { Image(systemName: "bookmark.slash") }
                                .buttonStyle(.borderless).help("Remove Saved Reference")
                                .accessibilityLabel("Remove \(reference.title) from Saved")
                        }
                    }.padding(.vertical, 4)
                }.listStyle(.inset)
            }
        }.padding(20).frame(width: 580, height: 520)
        .task { saved = initialSaved; await reading.refresh() }
    }
}
