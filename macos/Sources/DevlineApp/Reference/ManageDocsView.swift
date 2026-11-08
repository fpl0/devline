import SwiftUI

struct ManageDocsView: View {
    private enum Section: String, CaseIterable {
        case installed = "Installed"
        case available = "Available"
        case imported = "Imported"
    }

    @Environment(\.dismiss) private var dismiss
    @ObservedObject var model: WorkspaceModel
    var installRequested = false
    var requestedPackID: String?
    let openDocset: (DocPack) -> Void
    @State private var section: Section = .installed
    @State private var query = ""
    @State private var showingImporter = false
    @State private var updates: DocumentationUpdates?
    @State private var checking = false
    @State private var failure: String?
    @State private var detail: DocPack?
    @State private var removal: DocPack?
    @State private var detailJob: JobRecord?

    private var packs: [DocPack] { model.catalog?.docs ?? [] }
    private var installing: Bool {
        model.jobs.contains { $0.isActive && ["docs_install", "docs_prepare"].contains($0.kind) }
    }

    private var visiblePacks: [DocPack] {
        packs.filter { pack in
            switch section {
            case .installed: pack.installed && !pack.localOnly
            case .available: !pack.installed && !pack.localOnly
            case .imported: pack.installed && pack.id.hasPrefix("devdocs-")
            }
        }.filter { matches($0) }
    }

    private var downloads: [DocumentationRelease] {
        let onThisMac = Set(packs.map(\.reference))
        return (updates?.packs ?? []).flatMap(\.releases).filter {
            !onThisMac.contains($0.pack.reference) && matches($0.pack)
        }
    }

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            if showingImporter {
                DevDocsImportView(model: model, close: { showingImporter = false; section = .imported })
            } else {
                controls
                inventory
            }
        }
        .frame(width: 720, height: 510)
        .onAppear {
            if installRequested || !packs.contains(where: \.installed) { section = .available }
            if let requestedPackID { query = requestedPackID }
        }
        .task(id: model.libraryRevision) { await loadReleases(online: false) }
        .sheet(item: Binding(get: { detail.map { PackSelection(pack: $0) } }, set: { detail = $0?.pack })) {
            DocsetInfoView(pack: $0.pack)
        }
        .sheet(item: $detailJob) { JobDetailsView(job: $0, model: model) }
        .confirmationDialog("Remove documentation?", isPresented: Binding(
            get: { removal != nil }, set: { if !$0 { removal = nil } }
        ), titleVisibility: .visible, presenting: removal) { pack in
            Button("Remove", role: .destructive) {
                Task {
                    await model.mutate([CoreRequest("docs.remove", id: pack.id, version: pack.version)],
                                       label: "Removing \(name(pack))", completion: "Removed \(name(pack)).")
                }
            }
        } message: { pack in
            Text("\(name(pack)) \(displayDocVersion(pack.version)) will be removed from this Mac.")
        }
    }

    private var header: some View {
        HStack {
            Text("Manage Docs").font(.title2.weight(.semibold)).accessibilityAddTraits(.isHeader)
            Spacer()
            if !showingImporter {
                Button("Import from DevDocs…") { showingImporter = true; query = "" }
            }
            Button("Done") { dismiss() }.keyboardShortcut(.cancelAction)
        }.padding(18)
    }

    private var controls: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack {
                Picker("Documentation", selection: $section) {
                    ForEach(Section.allCases, id: \.self) { Text($0.rawValue).tag($0) }
                }.pickerStyle(.segmented).labelsHidden().fixedSize()
                Spacer()
                TextField("Filter docsets", text: $query).textFieldStyle(.roundedBorder).frame(maxWidth: 200)
            }
            if section == .available {
                HStack {
                    Button("Check for Downloads") { Task { await loadReleases(online: true) } }
                        .disabled(checking)
                    if checking { ProgressView().controlSize(.small) }
                    if let date = updates?.checkedAt {
                        Text("Checked \(Date(timeIntervalSince1970: date).formatted(date: .abbreviated, time: .shortened))")
                            .font(.caption).foregroundStyle(.secondary)
                    }
                    Spacer()
                }
                if let error = failure ?? updates?.error {
                    Label(error, systemImage: "exclamationmark.circle").font(.caption).foregroundStyle(.orange)
                }
            }
        }.padding(14)
    }

    @ViewBuilder private var inventory: some View {
        if section == .imported && visiblePacks.isEmpty {
            ContentUnavailableView(query.isEmpty ? "No Imported Docs" : "No Matching Imports", systemImage: "books.vertical")
        } else {
            List {
                if section == .installed && (query.isEmpty || "SwiftUI".localizedCaseInsensitiveContains(query)) {
                    LocalSDKReferenceView(model: model, openDocset: openDocset)
                }
                ForEach(visiblePacks, id: \.reference) { pack in
                    HStack(spacing: 12) {
                        packLabel(pack)
                        Spacer()
                        if pack.installed {
                            Button("Open") { openDocset(pack) }
                            Menu {
                                Button("Source Details…") { detail = pack }
                                if pack.active != true && !pack.localOnly {
                                    Button("Use as Default") {
                                        Task {
                                            await model.mutate(
                                                [CoreRequest("docs.distribution.rollback", id: pack.id, version: pack.version)],
                                                label: "Changing default release", completion: "Using \(name(pack)) \(displayDocVersion(pack.version)).")
                                        }
                                    }.disabled(model.operation != nil)
                                }
                                Button("Remove…", role: .destructive) { removal = pack }
                                    .disabled(model.operation != nil)
                            } label: { Image(systemName: "ellipsis.circle") }
                                .menuStyle(.borderlessButton).fixedSize()
                                .accessibilityLabel("Actions for \(name(pack)) \(displayDocVersion(pack.version))")
                        } else {
                            Button("Source Details…") { detail = pack }
                            Button("Install") { startInstall(pack, kind: "docs_prepare") }
                                .disabled(installing)
                                .accessibilityLabel("Install \(name(pack)) \(displayDocVersion(pack.version))")
                        }
                    }.controlSize(.small).padding(.vertical, 5)
                }
                if section == .available {
                    ForEach(downloads, id: \.pack.reference) { release in
                        DisclosureGroup {
                            Text(release.releaseNotes).font(.callout).textSelection(.enabled).padding(.vertical, 8)
                        } label: {
                            HStack(spacing: 12) {
                                packLabel(release.pack)
                                Spacer()
                                Text(storageSize(release.size)).font(.caption).foregroundStyle(.secondary)
                                Button("Source Details…") { detail = release.pack }
                                Button("Download") { startInstall(release.pack, kind: "docs_install") }
                                    .disabled(installing)
                                    .accessibilityLabel("Download \(name(release.pack)) \(displayDocVersion(release.pack.version))")
                            }.controlSize(.small)
                        }.padding(.vertical, 5)
                    }
                    if visiblePacks.isEmpty && downloads.isEmpty {
                        Text(query.isEmpty ? "Your available docsets are installed." : "No matching docsets.")
                            .foregroundStyle(.secondary)
                    }
                }
            }.listStyle(.inset)
        }
        if let job = model.jobs.first(where: { ["docs_install", "docs_prepare"].contains($0.kind) }),
           job.isActive || job.needsAttention {
            Divider()
            HStack {
                if job.isActive { ProgressView().controlSize(.small) }
                Text(job.error ?? job.stage).lineLimit(2)
                Spacer()
                Button("Details…") { detailJob = job }
            }.font(.caption).padding(12)
        }
    }

    private struct PackSelection: Identifiable {
        let pack: DocPack
        var id: String { pack.reference }
    }

    private func packLabel(_ pack: DocPack) -> some View {
        VStack(alignment: .leading, spacing: 3) {
            Text(name(pack)).font(.callout.weight(.medium))
            Text(displayDocVersion(pack.version) + (pack.localOnly ? " · Imported via DevDocs" : pack.active == true ? " · Default" : ""))
                .font(.caption).foregroundStyle(.secondary)
        }
    }

    private func name(_ pack: DocPack) -> String {
        pack.name.hasPrefix("DevDocs · ") ? String(pack.name.dropFirst("DevDocs · ".count)) : pack.name
    }

    private func matches(_ pack: DocPack) -> Bool {
        query.isEmpty || pack.reference == query || pack.id.localizedCaseInsensitiveContains(query) ||
            name(pack).localizedCaseInsensitiveContains(query) || displayDocVersion(pack.version).localizedCaseInsensitiveContains(query)
    }

    private func startInstall(_ pack: DocPack, kind: String) {
        Task { await model.startJob(CoreRequest("jobs.start", id: pack.id, version: pack.version, kind: kind), inspect: false) }
    }

    private func loadReleases(online: Bool) async {
        guard !checking else { return }
        checking = true
        failure = nil
        defer { checking = false }
        do { updates = try await Core.request(CoreRequest(online ? "docs.distribution.check" : "docs.distribution.status")) }
        catch is CancellationError {}
        catch { failure = error.localizedDescription }
    }
}
