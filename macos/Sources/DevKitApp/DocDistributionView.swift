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

struct DocDistributionView: View {
    @Environment(\.dismiss) private var dismiss
    @ObservedObject var model: WorkspaceModel
    @State private var updates: DocumentationUpdates?
    @State private var checking = false
    @State private var error: String?
    @State private var detail: DocPack?
    private var downloading: Bool { model.jobs.contains { $0.kind == "docs_install" && $0.isActive } }

    var body: some View {
        VStack(spacing: 0) {
            HStack(alignment: .top, spacing: 14) {
                VStack(alignment: .leading, spacing: 5) {
                    Text("Documentation Releases").font(.title2.weight(.semibold))
                    Text("Browse downloadable docsets, release notes, and installed versions.")
                        .font(.callout).foregroundStyle(.secondary)
                    if let date = updates?.checkedAt {
                        Text("Last checked \(Date(timeIntervalSince1970: date).formatted(date: .abbreviated, time: .shortened))")
                            .font(.caption).foregroundStyle(.secondary)
                    }
                }
                Spacer()
                if checking { ProgressView().controlSize(.small) }
                Button("Refresh Releases") { Task { await load(online: true) } }
                    .disabled(checking)
                Button("Done") { dismiss() }.keyboardShortcut(.cancelAction)
            }.padding(18)
            Divider()
            if let failure = error ?? updates?.error {
                Label(failure, systemImage: "exclamationmark.circle").font(.callout).foregroundStyle(.orange)
                    .frame(maxWidth: .infinity, alignment: .leading).padding(16)
            }
            if let updates, !updates.packs.isEmpty {
                List(updates.packs) { pack in
                    VStack(alignment: .leading, spacing: 10) {
                        HStack(spacing: 14) {
                            VStack(alignment: .leading, spacing: 4) {
                                Text(pack.name).font(.headline)
                                Text(pack.activeVersion.map { "Active: \(displayDocVersion($0))" } ?? "Not installed")
                                    .font(.caption).foregroundStyle(.secondary)
                            }
                            Spacer()
                            if pack.installedVersions.count > 1 {
                                Menu("Use Version") {
                                    ForEach(pack.installedVersions, id: \.self) { version in
                                        Button(displayDocVersion(version)) { Task { await activate(pack.id, version: version) } }
                                            .disabled(version == pack.activeVersion)
                                    }
                                }.fixedSize().help("Change the default version. Project pins are kept.")
                            }
                            if let latest = pack.latest {
                                Text(ByteCountFormatter.string(fromByteCount: latest.size, countStyle: .file)).font(.caption).foregroundStyle(.secondary)
                                if pack.state == "available" || pack.state == "installable" {
                                    Button(pack.installedVersions.contains(latest.pack.version) ? "Use Latest" : "Download and Use") {
                                        Task {
                                            if pack.installedVersions.contains(latest.pack.version) { await activate(pack.id, version: latest.pack.version) }
                                            else { await download(pack.id, version: latest.pack.version) }
                                        }
                                    }.controlSize(.small).disabled(downloading)
                                        .accessibilityLabel("\(pack.installedVersions.contains(latest.pack.version) ? "Use" : "Download and use") \(pack.name) \(latest.pack.version)")
                                } else { Label("Current", systemImage: "checkmark.circle").font(.caption).foregroundStyle(.secondary) }
                            } else { Text("Check online for releases").font(.caption).foregroundStyle(.secondary) }
                        }
                        if let latest = pack.latest {
                            DisclosureGroup("\(displayDocVersion(latest.pack.version)) · Release Notes") {
                                VStack(alignment: .leading, spacing: 9) {
                                    Text(latest.releaseNotes).font(.callout).textSelection(.enabled)
                                    Button("Coverage and Source…") { detail = latest.pack }.controlSize(.small)
                                    if pack.releases.count > 1 {
                                        ForEach(pack.releases.dropFirst(), id: \.pack.reference) { release in
                                            HStack {
                                                Text(displayDocVersion(release.pack.version)).font(.caption)
                                                Spacer()
                                                Text(ByteCountFormatter.string(fromByteCount: release.size, countStyle: .file)).font(.caption).foregroundStyle(.secondary)
                                                Button("Download and Use") {
                                                    Task { await download(pack.id, version: release.pack.version) }
                                                }.controlSize(.small).disabled(downloading || pack.installedVersions.contains(release.pack.version))
                                            }
                                        }
                                    }
                                }.padding(.top, 8)
                            }.font(.callout)
                        }
                    }.padding(.vertical, 8)
                }.listStyle(.inset)
            } else if checking { ProgressView().frame(maxWidth: .infinity, maxHeight: .infinity) }
            else { ContentUnavailableView("Check for Documentation", systemImage: "book.closed", description: Text("Connect once to see downloadable releases and their sizes. Installed articles remain available offline.")) }
            Text(downloading ? "Downloading documentation… Open Activity for progress." : "Find download progress and logs in Activity.")
                .font(.caption).foregroundStyle(.secondary).padding(12)
        }
        .frame(width: 740, height: 560)
        .task { await load(online: false) }
        .onChange(of: model.libraryRevision) { _, _ in Task { await load(online: false) } }
        .sheet(isPresented: Binding(get: { detail != nil }, set: { if !$0 { detail = nil } })) {
            if let detail { DocsetInfoView(pack: detail) }
        }
    }
    private func download(_ id: String, version: String) async {
        await model.startJob(CoreRequest("jobs.start", id: id, version: version, kind: "docs_install"), inspect: false)
    }
    private func load(online: Bool) async {
        guard !checking else { return }
        checking = true; error = nil
        defer { checking = false }
        do { updates = try await Core.request(CoreRequest(online ? "docs.distribution.check" : "docs.distribution.status")) }
        catch is CancellationError {}
        catch { self.error = error.localizedDescription }
    }
    private func activate(_ id: String, version: String) async {
        _ = await model.mutate([CoreRequest("docs.distribution.rollback", id: id, version: version)], label: "Changing documentation version", completion: "Using \(id) \(version) by default. Project pins are kept.")
        await load(online: false)
    }
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
            if pack.localOnly {
                Label("Imported from this Mac’s Xcode SDK. Recreate on another Mac using its own Xcode installation.", systemImage: "desktopcomputer")
                    .font(.callout).foregroundStyle(.secondary)
            }
            LabeledContent("Snapshot", value: displayDocVersion(pack.version))
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
            if let url = safeExternalURL(pack.sourceUrl) { Link("Official Source", destination: url) }
        }.padding(22).frame(width: 600, height: 480).textSelection(.enabled)
    }
}
