import AppKit
import SwiftUI
import UniformTypeIdentifiers

struct LibraryView: View {
    private struct ReadinessSelection: Equatable {
        let path: String?
        let docs, starters, images: Set<String>
        let libraryRevision: Int
        let completedJobs: [String]
    }

    @ObservedObject var model: WorkspaceModel
    @ObservedObject var infrastructure: InfrastructureLibrary
    let catalog: Catalog
    let projectPath: String?
    @State private var docs: Set<String> = []
    @State private var starters: Set<String> = []
    @State private var images: Set<String> = []
    @State private var showAllVersions = false
    @State private var checkedSelection: ReadinessSelection?
    @State private var checkedStatus: ToolboxStatus?
    @State private var selectionRevision = 0
    @State private var checking = false
    @State private var error: String?
    @State private var removePack: DocPack?
    @State private var infoPack: DocPack?
    @State private var showingReleases = false
    @State private var importPath: String?

    private var readinessSelection: ReadinessSelection {
        ReadinessSelection(path: projectPath, docs: docs, starters: starters, images: images,
                           libraryRevision: model.libraryRevision, completedJobs: model.completedJobIDs)
    }
    private var result: ToolboxStatus? { checkedSelection == readinessSelection ? checkedStatus : nil }
    private var selectionIsEmpty: Bool { docs.isEmpty && starters.isEmpty && images.isEmpty }
    private var defaultDocs: [DocPack] {
        Dictionary(grouping: catalog.docs, by: \.id).values.compactMap { releases in
            releases.first { $0.active == true } ?? releases.first
        }
    }
    private var defaultStarters: [Starter] {
        Dictionary(grouping: catalog.templates, by: \.id).values.compactMap { releases in
            releases.first { $0.verified } ?? releases.first { $0.isInstalled } ?? releases.first
        }
    }
    private var visibleDocs: [DocPack] {
        let defaults = Set(defaultDocs.map(\.reference))
        let checked = Set((result?.library.docs ?? []).map(\.reference))
        return catalog.docs.filter {
            showAllVersions || defaults.contains($0.reference) || docs.contains($0.reference) || checked.contains($0.reference)
        }.sorted { releasePrecedes($0.name, $0.reference, $1.name, $1.reference) }
    }
    private var unlistedDocs: [OfflineStatus.Doc] {
        let listed = Set(catalog.docs.map(\.reference))
        return (result?.library.docs ?? []).filter { !listed.contains($0.reference) }
    }
    private var visibleStarters: [Starter] {
        let defaults = Set(defaultStarters.map(\.reference))
        return catalog.templates.filter {
            showAllVersions || defaults.contains($0.reference) || starters.contains($0.reference)
        }.sorted { releasePrecedes($0.name, $0.reference, $1.name, $1.reference) }
    }
    private var installRequests: [CoreRequest] {
        catalog.docs.filter { docs.contains($0.reference) && !$0.installed }
            .map { CoreRequest("docs.install", id: $0.id, version: $0.version) } +
        catalog.templates.filter { starters.contains($0.reference) && !$0.isInstalled }
            .map { CoreRequest("templates.install", id: $0.reference) }
    }
    private var portableDocs: Set<String> {
        docs.subtracting(catalog.docs.filter(\.localOnly).map(\.reference))
    }
    private var archivedImages: Set<String> { Set(infrastructure.archived.map(\.reference)) }
    private var hasInstalledItems: Bool {
        catalog.docs.contains(where: \.installed) || catalog.templates.contains(where: \.isInstalled) || !archivedImages.isEmpty
    }

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            selectionControls
            selectionList
            Divider()
            footer
        }
        .task(id: model.completedJobIDs) { await infrastructure.refresh() }
        .onChange(of: readinessSelection) { _, _ in invalidateReadiness() }
        .onChange(of: catalog.docs.map(\.reference)) { _, references in docs.formIntersection(references) }
        .onChange(of: catalog.templates.map(\.reference)) { _, references in starters.formIntersection(references) }
        .sheet(isPresented: $showingReleases) { DocDistributionView(model: model) }
        .sheet(isPresented: Binding(get: { infoPack != nil }, set: { if !$0 { infoPack = nil } })) {
            if let infoPack { DocsetInfoView(pack: infoPack) }
        }
        .sheet(isPresented: Binding(get: { importPath != nil }, set: { if !$0 { importPath = nil } })) {
            if let importPath { BundleImportView(path: importPath, model: model) }
        }
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
        VStack(alignment: .leading, spacing: 10) {
            Text("Install documentation and starters, check offline readiness, and transfer portable bundles.")
                .font(.callout).foregroundStyle(.secondary)
            HStack(spacing: 10) {
                Button("Manage Releases…") { showingReleases = true }
                    .help("Browse downloadable docsets, release notes, and installed versions")
                Button("Import Bundle…", action: chooseImport)
                Menu("Export Selected…") {
                    Button("Documentation and Starters…", action: exportBundle).disabled(portableDocs.isEmpty && starters.isEmpty)
                    Button("Image Archives…", action: exportImages).disabled(images.isEmpty)
                }.disabled(portableDocs.isEmpty && starters.isEmpty && images.isEmpty)
                Spacer()
                Button {
                    invalidateReadiness()
                    Task {
                        await model.refresh()
                        await infrastructure.refresh()
                    }
                } label: {
                    Image(systemName: "arrow.clockwise")
                }
                .help("Refresh library")
                .accessibilityLabel("Refresh library")
                .disabled(model.operation != nil || checking)
            }
        }.padding(18)
    }

    private var selectionControls: some View {
        HStack {
            Text("Selected · Docsets \(docs.count) · Starters \(starters.count) · Images \(images.count)")
                .foregroundStyle(.secondary)
            Spacer()
            Menu("Selection") {
                Button("Select Installed") {
                    docs.formUnion(catalog.docs.filter(\.installed).map(\.reference))
                    starters.formUnion(catalog.templates.filter(\.isInstalled).map(\.reference))
                    images.formUnion(archivedImages)
                }.disabled(!hasInstalledItems)
                Button("Clear Selection") {
                    docs.removeAll()
                    starters.removeAll()
                    images.removeAll()
                }.disabled(selectionIsEmpty)
            }.fixedSize().disabled(model.operation != nil)
            Toggle("Show All Versions", isOn: $showAllVersions).toggleStyle(.checkbox)
        }.font(.caption).padding(.horizontal, 18).padding(.vertical, 8)
    }

    private var selectionList: some View {
        ControlList {
            ControlListSection("From This Mac") {
                ControlListRow { LocalSDKReferenceView(model: model) }
            }
            if let runtimes = result?.library.runtimes, !runtimes.isEmpty {
                ControlListSection("Project Toolchains") {
                    ForEach(runtimes) { runtime in
                        ControlListRow {
                            HStack {
                                title(runtime.name + (runtime.version.map { " · \($0)" } ?? ""), detail: runtime.detail)
                                Spacer()
                                readiness(runtime.readyOffline, detail: runtime.detail)
                            }
                        }
                    }
                }
            }
            ControlListSection("Documentation") {
                ForEach(unlistedDocs, id: \.reference) { doc in
                    ControlListRow {
                        HStack {
                            let name = catalog.docs.first { $0.id == doc.id }?.name ?? doc.id
                            title(name + (doc.version.map { " · \($0)" } ?? ""), detail: doc.detail ?? "")
                            Spacer()
                            readiness(doc.readyOffline, detail: doc.detail)
                        }
                    }
                }
                ForEach(visibleDocs, id: \.reference) { pack in
                    ControlListRow { docRow(pack) }
                }
            }
            ControlListSection("Starter Templates") {
                ForEach(visibleStarters, id: \.reference) { starter in
                    ControlListRow { starterRow(starter) }
                }
            }
            ControlListSection("Infrastructure Images") {
                if let error = infrastructure.error {
                    Text(error).font(.caption).foregroundStyle(.orange)
                }
                ForEach(infrastructure.catalog?.services ?? [], id: \.reference) { service in
                    ControlListRow {
                        HStack {
                            Toggle(isOn: selection(service.reference, in: $images)) {
                                title(service.name, detail: service.version)
                            }.toggleStyle(.checkbox)
                            Spacer()
                            if let checked = result?.images.first(where: { $0.reference == service.reference }) {
                                readiness(checked.readyOffline, detail: checked.status)
                            } else {
                                let image = infrastructure.image(service.reference)
                                rowStatus(image?.verifiedArchive == true ? "Archived" : image?.archived == true ? "Verify Archive" : "Not Archived")
                            }
                        }
                    }
                }
            }
        }
    }

    private func docRow(_ pack: DocPack) -> some View {
        HStack(spacing: 10) {
            Toggle(isOn: selection(pack.reference, in: $docs)) {
                title(pack.name, detail: pack.description)
            }.toggleStyle(.checkbox)
                .accessibilityLabel("\(pack.name) \(pack.version)")
            Spacer()
            if let checked = result?.library.docs.first(where: { $0.reference == pack.reference }) {
                readiness(checked.readyOffline, detail: checked.detail)
            } else {
                rowStatus(pack.installed ? "Installed" : "Not Installed")
            }
            if pack.localOnly {
                rowStatus("This Mac Only").help("Excluded from portable bundles")
            }
            Text(pack.version).font(.callout.monospacedDigit()).foregroundStyle(.secondary)
            Button { infoPack = pack } label: { Image(systemName: "info.circle") }
                .buttonStyle(.borderless).help("Coverage and source for \(pack.name)")
                .accessibilityLabel("Coverage and source for \(pack.name) \(pack.version)")
            if pack.installed {
                Button { removePack = pack } label: { Image(systemName: "ellipsis.circle") }
                    .buttonStyle(.borderless).help("Remove \(pack.name) \(pack.version)")
                    .accessibilityLabel("Remove \(pack.name) \(pack.version)")
                    .disabled(model.operation != nil)
            }
        }
    }

    private func starterRow(_ starter: Starter) -> some View {
        let checked = result?.library.starters.first { $0.template?.reference == starter.reference || $0.id == starter.reference }
        return VStack(alignment: .leading, spacing: 6) {
            HStack(spacing: 10) {
                Toggle(isOn: selection(starter.reference, in: $starters)) {
                    title(starter.name, detail: "\(displayStack(starter.stack)) · \(starter.target)")
                }.toggleStyle(.checkbox)
                    .accessibilityLabel("\(starter.name) \(starter.version)")
                Spacer()
                if let checked {
                    readiness(checked.readyOffline, detail: checked.error)
                } else {
                    rowStatus(starter.isInstalled ? "Installed" : "Not Installed")
                }
                Text(starter.version).font(.callout.monospacedDigit()).foregroundStyle(.secondary)
                if starter.isInstalled {
                    Button("Verify Offline") { Task { await model.startJob(CoreRequest("jobs.start", id: starter.reference, kind: "verify")) } }
                        .buttonStyle(.bordered).controlSize(.small)
                        .accessibilityLabel("Verify \(starter.name) \(starter.version) Offline")
                }
            }
            ForEach((checked?.checks ?? []).filter { !$0.ok }, id: \.name) { check in
                Text("\(check.name): \(check.detail)").font(.caption).foregroundStyle(.orange)
            }
        }
    }

    private var footer: some View {
        HStack(spacing: 10) {
            if checking { ProgressView().controlSize(.small) }
            if let error {
                Text(error).foregroundStyle(.orange).textSelection(.enabled)
            } else if let result {
                Label(result.readyOffline ? "Ready Offline" : "Needs Preparation",
                      systemImage: result.readyOffline ? "checkmark.circle" : "exclamationmark.circle")
                    .foregroundStyle(result.readyOffline ? Color.green : .orange)
                if result.projectDependencies.state == "unverified", let detail = result.projectDependencies.detail {
                    Text(detail).foregroundStyle(.secondary)
                }
            } else {
                Text(projectPath == nil ? "Select items to check their offline readiness." : "Readiness checks include the open project.")
                    .foregroundStyle(.secondary)
            }
            Spacer()
            Button("Check Readiness") { Task { await check() } }
                .disabled(checking || model.operation != nil || (selectionIsEmpty && projectPath == nil))
            let requests = installRequests
            Button(requests.isEmpty ? "Install Selected" : "Install Selected (\(requests.count))") {
                invalidateReadiness()
                Task {
                    await model.mutate(requests, label: "Installing selected packs",
                                       completion: "Installed \(requests.count) \(requests.count == 1 ? "pack" : "packs").")
                }
            }.buttonStyle(.borderedProminent)
                .disabled(requests.isEmpty || model.operation != nil || checking)
        }.font(.callout).padding(18)
    }

    private func title(_ name: String, detail: String) -> some View {
        VStack(alignment: .leading, spacing: 3) {
            Text(name).font(.body.weight(.medium))
            Text(detail).font(.caption).foregroundStyle(.secondary).lineLimit(2)
        }
    }

    private func rowStatus(_ label: String) -> some View {
        Text(label).font(.caption).foregroundStyle(.secondary)
    }

    private func readiness(_ ready: Bool, detail: String?) -> some View {
        Label(ready ? "Verified" : "Needs Preparation", systemImage: ready ? "checkmark.circle" : "exclamationmark.circle")
            .font(.caption).foregroundStyle(ready ? Color.green : .orange).help(detail ?? "")
    }

    private func releasePrecedes(_ name: String, _ reference: String, _ otherName: String, _ otherReference: String) -> Bool {
        let order = name.localizedStandardCompare(otherName)
        if order != .orderedSame { return order == .orderedAscending }
        return reference.compare(otherReference, options: .numeric) == .orderedDescending
    }

    private func selection(_ reference: String, in values: Binding<Set<String>>) -> Binding<Bool> {
        Binding(get: { values.wrappedValue.contains(reference) }, set: {
            if $0 { values.wrappedValue.insert(reference) } else { values.wrappedValue.remove(reference) }
        })
    }

    private func invalidateReadiness() {
        selectionRevision += 1
        checkedSelection = nil
        error = nil
    }

    private func check() async {
        guard !checking else { return }
        let selection = readinessSelection
        invalidateReadiness()
        let revision = selectionRevision
        checking = true
        defer { checking = false }
        do {
            let result: ToolboxStatus = try await Core.request(CoreRequest("toolbox.check", path: selection.path,
                docs: selection.docs.sorted(), starters: selection.starters.sorted(), images: selection.images.sorted()))
            guard selectionRevision == revision, readinessSelection == selection else { return }
            checkedStatus = result
            checkedSelection = selection
        } catch is CancellationError {} catch {
            if selectionRevision == revision, readinessSelection == selection { self.error = error.localizedDescription }
        }
    }

    private func chooseImport() {
        let panel = NSOpenPanel()
        panel.title = "Inspect Offline Bundle"
        panel.prompt = "Inspect"
        panel.allowedContentTypes = [UTType(filenameExtension: "devlinebundle") ?? .data]
        if panel.runModal() == .OK { importPath = panel.url?.path(percentEncoded: false) }
    }

    private func exportBundle() {
        let panel = NSSavePanel()
        panel.title = "Export Offline Bundle"
        panel.nameFieldStringValue = "Devline-offline.devlinebundle"
        panel.allowedContentTypes = [UTType(filenameExtension: "devlinebundle") ?? .data]
        if panel.runModal() == .OK, let path = panel.url?.path(percentEncoded: false) {
            Task { await model.startJob(CoreRequest("jobs.start", path: path, kind: "bundle_export", docs: portableDocs.sorted(), starters: starters.sorted())) }
        }
    }

    private func exportImages() {
        let panel = NSSavePanel()
        panel.title = "Export Image Archives"
        panel.nameFieldStringValue = "Devline-images.devlineimages"
        if panel.runModal() == .OK, let path = panel.url?.path(percentEncoded: false) {
            Task { await model.startJob(CoreRequest("jobs.start", path: path, kind: "infra_export", images: images.sorted())) }
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
