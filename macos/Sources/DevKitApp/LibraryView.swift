import SwiftUI

struct LibraryView: View {
    @ObservedObject var model: WorkspaceModel
    @State private var selections: Set<Choice> = []
    @State private var removePack: DocPack?
    @State private var infoPack: DocPack?
    @State private var showingReleases = false

    var body: some View {
        VStack(spacing: 0) {
            if let catalog = model.catalog {
                header
                Divider()
                List {
                    Section("From This Mac") {
                        LocalSDKReferenceView(model: model)
                    }
                    Section("Documentation") {
                        ForEach(catalog.docs, id: \.reference) { pack in
                            choiceRow(.doc(pack), name: pack.name, version: pack.version,
                                      detail: pack.description, installed: pack.installed)
                        }
                    }
                    Section("Starter Templates") {
                        ForEach(catalog.templates, id: \.reference) { starter in
                            choiceRow(.starter(starter), name: starter.name, version: starter.version,
                                      detail: "\(displayStack(starter.stack)) · \(starter.target)", installed: starter.isInstalled)
                        }
                    }
                }
                .listStyle(.inset)
                .onChange(of: availableChoices(in: catalog)) { _, choices in selections.formIntersection(choices) }
            } else {
                ContentUnavailableView {
                    Label("Library Unavailable", systemImage: "tray")
                } description: {
                    Text("Load the catalog to see available doc packs and templates.")
                } actions: {
                    Button("Reload Catalog") { Task { await model.refresh() } }
                }
            }
        }
        .sheet(isPresented: Binding(get: { infoPack != nil }, set: { if !$0 { infoPack = nil } })) {
            if let infoPack { DocsetInfoView(pack: infoPack) }
        }
        .sheet(isPresented: $showingReleases) { DocDistributionView(model: model) }
        .confirmationDialog("Remove this doc pack?", isPresented: Binding(
            get: { removePack != nil }, set: { if !$0 { removePack = nil } }
        ), titleVisibility: .visible, presenting: removePack) { pack in
            Button("Remove \(pack.name)", role: .destructive) {
                Task {
                    await model.mutate([CoreRequest("docs.remove", id: pack.id, version: pack.version)],
                                       label: "Removing \(pack.name)", completion: "Removed \(pack.name) \(pack.version).")
                }
            }
        } message: { pack in
            Text("\(pack.name) \(pack.version) will no longer appear in offline search.")
        }
    }

    private var header: some View {
        HStack(alignment: .center, spacing: 12) {
            VStack(alignment: .leading, spacing: 4) {
                Text("Docset Library").font(.title2.weight(.semibold))
                Text("Select doc packs and starters from any stack to install together.")
                    .font(.callout).foregroundStyle(.secondary)
            }
            Spacer()
            Button("Manage Releases…") { showingReleases = true }
                .help("Browse downloadable docsets, release notes, and installed versions")
            Button {
                Task { await model.refresh() }
            } label: {
                Image(systemName: "arrow.clockwise")
            }
            .help("Refresh library")
            .accessibilityLabel("Refresh library")
            .disabled(model.operation != nil)
            Button(selections.isEmpty ? "Install Selected" : "Install Selected (\(selections.count))") {
                let count = selections.count
                let requests = selections.sorted { $0.sortKey < $1.sortKey }.map(\.request)
                Task {
                    await model.mutate(requests, label: "Installing selected packs",
                                       completion: "Installed \(count) \(count == 1 ? "pack" : "packs").")
                }
            }
            .buttonStyle(.borderedProminent)
            .disabled(selections.isEmpty || model.operation != nil)
        }
        .padding(18)
    }

    private func choiceRow(_ choice: Choice, name: String, version: String, detail: String, installed: Bool) -> some View {
        HStack(spacing: 10) {
            if installed {
                Image(systemName: "checkmark.circle.fill").foregroundStyle(.green)
                    .frame(width: 16)
                    .accessibilityHidden(true)
                title(name, detail: detail)
            } else {
                Toggle(isOn: Binding(
                    get: { selections.contains(choice) },
                    set: { if $0 { selections.insert(choice) } else { selections.remove(choice) } }
                )) {
                    title(name, detail: detail)
                }
                .toggleStyle(.checkbox)
                .disabled(model.operation != nil)
            }
            Spacer(minLength: 12)
            if installed { Text("Installed").font(.caption).foregroundStyle(.secondary) }
            if case .doc(let pack) = choice, pack.localOnly {
                Text("This Mac Only").font(.caption).foregroundStyle(.secondary)
            }
            Text(version).font(.callout.monospacedDigit()).foregroundStyle(.secondary)
                .frame(minWidth: 54, alignment: .trailing)
            if case .doc(let pack) = choice {
                Button { infoPack = pack } label: { Image(systemName: "info.circle") }
                    .buttonStyle(.borderless).help("Coverage and source for \(name)")
                    .accessibilityLabel("Coverage and source for \(name) \(version)")
            }
            if installed, case .doc(let pack) = choice {
                Button {
                    removePack = pack
                } label: {
                    Image(systemName: "ellipsis.circle")
                }
                .buttonStyle(.borderless)
                .help("Remove \(name) \(version)")
                .accessibilityLabel("Remove \(name) \(version)")
                .disabled(model.operation != nil)
            }
        }
        .padding(.vertical, 5)
    }

    private func title(_ name: String, detail: String) -> some View {
        VStack(alignment: .leading, spacing: 3) {
            Text(name).font(.body.weight(.medium))
            Text(detail).font(.caption).foregroundStyle(.secondary).lineLimit(2)
        }
    }

    private func availableChoices(in catalog: Catalog) -> Set<Choice> {
        Set(catalog.docs.filter { !$0.installed }.map(Choice.doc) +
            catalog.templates.filter { !$0.isInstalled }.map(Choice.starter))
    }

    private enum Choice: Hashable {
        case doc(DocPack)
        case starter(Starter)

        var request: CoreRequest {
            switch self {
            case .doc(let pack): CoreRequest("docs.install", id: pack.id, version: pack.version)
            case .starter(let starter): CoreRequest("templates.install", id: starter.reference)
            }
        }

        var sortKey: String {
            switch self {
            case .doc(let pack): "0:\(pack.reference)"
            case .starter(let starter): "1:\(starter.reference)"
            }
        }
    }
}


struct LocalSDKReferenceView: View {
    @ObservedObject var model: WorkspaceModel
    @State private var status: LocalDocumentationStatus?
    @State private var failure: String?
    @State private var checking = false
    @State private var loadID = UUID()

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Label("SwiftUI · Apple API Reference", systemImage: "swift").font(.headline)
                Spacer()
                Button { Task { await load() } } label: {
                    Image(systemName: "arrow.clockwise")
                }.help("Check the selected Xcode SDK")
                    .accessibilityLabel("Check the selected Xcode SDK").disabled(checking)
                Button(status?.installed == true ? "Refresh from Xcode" : "Prepare from Xcode") {
                    Task { await model.startJob(CoreRequest("jobs.start", id: "swiftui", kind: "docs_prepare")) }
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
