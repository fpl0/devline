import SwiftUI

struct DocumentationRelease: Decodable, Sendable {
    let pack: DocPack
    let releaseNotes, url, sha256: String
    let size: Int64
}

struct DocumentationUpdates: Decodable, Sendable {
    struct Pack: Decodable, Identifiable, Sendable {
        let id, name, stack, state: String
        let activeVersion: String?
        let installedVersions: [String]
        let releases: [DocumentationRelease]
        let latest: DocumentationRelease?
    }
    let channelUrl, error: String?
    let checkedAt: Double?
    let packs: [Pack]
}

struct DocsetInfoView: View {
    let pack: DocPack
    @Environment(\.dismiss) private var dismiss
    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            HStack {
                Text(pack.name).font(.title2.weight(.semibold))
                Spacer()
                Button("Done") { dismiss() }.keyboardShortcut(.cancelAction)
            }
            Text(pack.description).foregroundStyle(.secondary)
            if pack.id == "swiftui" && pack.localOnly {
                Label("Imported from this Mac’s Xcode SDK. Recreate on another Mac using its own Xcode installation.", systemImage: "desktopcomputer")
                    .font(.callout).foregroundStyle(.secondary)
            } else if pack.localOnly {
                Label("Imported for this Mac. Excluded from portable bundles.", systemImage: "desktopcomputer")
                    .font(.callout).foregroundStyle(.secondary)
            }
            LabeledContent("Snapshot", value: displayDocVersion(pack.version))
            if pack.id.hasPrefix("devdocs-") && pack.version.contains("+devdocs.") {
                LabeledContent("Snapshot ID") {
                    Text(pack.version).font(.caption.monospaced()).lineLimit(2)
                        .frame(maxWidth: 405, alignment: .trailing)
                }
            }
            LabeledContent("Upstream Version", value: pack.upstreamVersion ?? "Not recorded for this snapshot")
            LabeledContent("Snapshot Date", value: pack.snapshotDate ?? "Not recorded for this snapshot")
            ScrollView {
                VStack(alignment: .leading, spacing: 10) {
                    Text("Included").font(.headline)
                    ForEach(pack.coverage ?? ["Coverage was not recorded for this older snapshot."], id: \.self) { Text($0).font(.callout) }
                    Divider()
                    Text("Known Omissions").font(.headline)
                    ForEach(pack.omissions ?? ["Omissions were not recorded for this older snapshot."], id: \.self) { Text($0).font(.callout).foregroundStyle(.secondary) }
                }.frame(maxWidth: .infinity, alignment: .leading)
            }
            if let url = safeExternalURL(pack.sourceUrl) {
                Link(pack.id.hasPrefix("devdocs-") ? "Source" : "Official Source", destination: url)
            }
        }.padding(22).frame(width: 600, height: 480).textSelection(.enabled)
    }
}

struct LocalSDKReferenceView: View {
    @ObservedObject var model: WorkspaceModel
    let openDocset: (DocPack) -> Void
    @State private var status: LocalDocumentationStatus?
    @State private var failure: String?
    @State private var checking = false
    @State private var loadID = UUID()

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Label("SwiftUI · Apple API Reference", systemImage: "swift").font(.headline)
                Spacer()
                if let pack = model.catalog?.docs.first(where: { $0.id == "swiftui" && $0.installed && $0.active == true }) {
                    Button("Open") { openDocset(pack) }
                }
                Button { Task { await load() } } label: {
                    Image(systemName: "arrow.clockwise")
                }.help("Check the selected Xcode SDK")
                    .accessibilityLabel("Check the selected Xcode SDK").disabled(checking)
                Button(status?.installed == true ? "Refresh from Xcode" : "Prepare from Xcode") {
                    Task { await model.startJob(CoreRequest("jobs.start", id: "swiftui", kind: "docs_prepare"), inspect: false) }
                }.disabled(status?.available != true || model.jobs.contains { $0.isActive && $0.kind == "docs_prepare" })
            }
            if let sdk = status?.sdkVersion {
                Text("macOS SDK \(sdk)" + (status?.count.map { " · \($0.formatted()) indexed symbols" } ?? " · Ready to import"))
                    .font(.caption).foregroundStyle(.secondary)
            }
            Text(failure ?? status?.detail ?? "Checking the selected Xcode SDK…")
                .font(.callout).foregroundStyle(.secondary)
            Text("Apple’s API declarations, documentation comments, and examples from your installed SDK. Website tutorials are separate. Stored on this Mac; excluded from exports.")
                .font(.caption).foregroundStyle(.secondary)
        }.padding(.vertical, 6)
        .task(id: model.libraryRevision) { await load() }
    }

    private func load() async {
        let token = UUID()
        loadID = token
        checking = true
        defer { if loadID == token { checking = false } }
        do {
            let next: LocalDocumentationStatus = try await Core.request(CoreRequest("docs.local.status", id: "swiftui"))
            guard loadID == token, !Task.isCancelled else { return }
            status = next
            failure = nil
        } catch is CancellationError {
        } catch {
            guard loadID == token, !Task.isCancelled else { return }
            failure = error.localizedDescription
        }
    }
}
