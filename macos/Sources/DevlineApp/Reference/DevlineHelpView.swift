import AppKit
import SwiftUI

struct DevlineHelpView: View {
    private struct Guide: Decodable, Sendable {
        let id, title, html, sourceUrl: String
    }

    private struct Topic: Decodable, Identifiable, Sendable {
        let id, title: String
    }

    @Environment(\.dismiss) private var dismiss
    @State private var guideID: String
    @State private var guide: Guide?
    @State private var topics: [Topic] = []
    @State private var anchor: String?
    @State private var navigation = UUID()
    @State private var failure: String?

    init(id: String) {
        _guideID = State(initialValue: id)
    }

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                VStack(alignment: .leading, spacing: 3) {
                    Label("Devline Help", systemImage: "questionmark.circle").font(.headline)
                    Text("Publisher links open in your browser.").font(.caption).foregroundStyle(.secondary)
                }
                Spacer()
                if !topics.isEmpty {
                    Menu("Guide Topics") {
                        ForEach(topics) { topic in
                            Button(topic.title) { open(topic.id) }
                        }
                    }.fixedSize()
                }
                Button("Done") { dismiss() }.keyboardShortcut(.cancelAction)
            }.padding(16)
            Divider()
            if let guide, guide.id == guideID {
                DevlineHelpArticle(html: guide.html, title: guide.title, sourceURL: guide.sourceUrl,
                                   anchor: anchor, navigation: navigation, navigate: follow)
                    .id(guide.id)
            } else if let failure {
                ContentUnavailableView("Guide Unavailable", systemImage: "questionmark.circle",
                                       description: Text(failure))
            } else {
                ProgressView("Opening guide…").frame(maxWidth: .infinity, maxHeight: .infinity)
            }
        }
        .frame(width: 780, height: 580)
        .task { topics = (try? await Core.request(CoreRequest("guide.list"))) ?? [] }
        .task(id: guideID) { await load(guideID) }
    }

    private func load(_ id: String) async {
        failure = nil
        do {
            let result: Guide = try await Core.request(CoreRequest("guide.show", id: id))
            guard !Task.isCancelled, guideID == id else { return }
            guide = result
        } catch is CancellationError {
        } catch {
            if guideID == id { failure = error.localizedDescription }
        }
    }

    private func open(_ id: String, anchor: String? = nil) {
        if guideID != id { guide = nil; failure = nil }
        guideID = id
        self.anchor = anchor
        navigation = UUID()
    }

    private func follow(_ url: URL) {
        let prefix = "/fpl0/devline/blob/main/data/help/infrastructure/"
        if url.scheme == "https", url.host == "github.com", url.path.hasPrefix(prefix),
           url.pathExtension == "md", url.deletingLastPathComponent().path == String(prefix.dropLast()) {
            open(url.deletingPathExtension().lastPathComponent, anchor: url.fragment?.removingPercentEncoding)
        } else {
            NSWorkspace.shared.open(url)
        }
    }
}

private struct DevlineHelpArticle: View {
    let html, title, sourceURL: String
    let anchor: String?
    let navigation: UUID
    let navigate: (URL) -> Void
    @State private var request: ReaderRequest?

    var body: some View {
        OfflineArticle(html: html, title: title, sourceURL: sourceURL, packID: "", scale: 1,
                       request: request, report: { _, _ in }, reportSections: { _ in },
                       navigate: navigate, reportPosition: { _, _, _, _ in })
            .task(id: navigation) {
                let action = anchor.map { ReaderRequest.Action.jump(ReaderSection(title: "", target: .anchor($0))) }
                    ?? .restore(0, anchor: nil, offset: nil)
                request = ReaderRequest(action: action)
            }
    }
}
