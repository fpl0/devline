import AppKit
import SwiftUI
import UniformTypeIdentifiers

struct OfflineStatus: Decodable, Sendable {
    struct Doc: Decodable, Identifiable, Sendable {
        let id: String
        let version, detail: String?
        let readyOffline: Bool
    }
    struct StarterResult: Decodable, Sendable {
        let id, error: String?
        let template: Starter?
        let checks: [Readiness.Check]?
        let readyOffline: Bool
    }
    struct Runtime: Decodable, Identifiable, Sendable {
        let id, name, detail: String
        let version: String?
        let readyOffline: Bool
    }
    let readyOffline: Bool
    let status: String
    let docs: [Doc]
    let starters: [StarterResult]
    let runtimes: [Runtime]?
}

struct BundleInspection: Decodable, Sendable {
    struct Item: Decodable, Identifiable, Sendable { let id, name, version: String }
    struct Problem: Decodable, Sendable { let id, reason: String }
    let compatible, requiresTrust: Bool
    let problems: [Problem]
    let warnings: [Problem]?
    let docs, starters: [Item]
    let sizeBytes, fileCount: Int
}

struct OfflineLibraryView: View {
    private struct ReadinessSelection: Equatable {
        let path: String?
        let docs, starters, images: Set<String>
        let libraryRevision: Int
        let completedJobs: [String]
    }

    @ObservedObject var model: WorkspaceModel
    @ObservedObject var infrastructure: InfrastructureLibrary
    var projectPath: String?
    @State private var docs: Set<String> = []
    @State private var starters: Set<String> = []
    @State private var images: Set<String> = []
    @State private var showAllVersions = false
    @State private var checkedSelection: ReadinessSelection?
    @State private var checkedStatus: ToolboxStatus?
    @State private var selectionRevision = 0
    @State private var checking = false
    @State private var error: String?
    @State private var importPath: String?
    @State private var seeded = false
    @State private var seededProject = false
    private var readinessSelection: ReadinessSelection {
        ReadinessSelection(path: projectPath, docs: docs, starters: starters, images: images, libraryRevision: model.libraryRevision,
                           completedJobs: model.jobs.filter { $0.state == "succeeded" }.map(\.id))
    }
    private var status: OfflineStatus? { checkedSelection == readinessSelection ? checkedStatus?.library : nil }
    private var allReady: Bool { checkedSelection == readinessSelection && checkedStatus?.readyOffline == true }
    private var visibleDocs: [DocPack] {
        let all = model.catalog?.docs ?? []
        let releases = showAllVersions ? all : Dictionary(grouping: all, by: \.id).values.compactMap { releases in
            releases.first { $0.active == true } ?? releases.max { $0.version.compare($1.version, options: .numeric) == .orderedAscending }
        }
        return releases.sorted { releasePrecedes($0.name, $0.reference, $1.name, $1.reference) }
    }
    private var visibleStarters: [Starter] {
        let all = model.catalog?.templates ?? []
        let releases = showAllVersions ? all : Dictionary(grouping: all, by: \.id).values.compactMap { releases in
            releases.filter(\.verified).max { $0.version.compare($1.version, options: .numeric) == .orderedAscending } ?? releases.first
        }
        return releases.sorted { releasePrecedes($0.name, $0.reference, $1.name, $1.reference) }
    }
    private func releasePrecedes(_ name: String, _ reference: String, _ otherName: String, _ otherReference: String) -> Bool {
        let order = name.localizedStandardCompare(otherName)
        if order != .orderedSame { return order == .orderedAscending }
        return reference.compare(otherReference, options: .numeric) == .orderedDescending
    }
    private var portableDocs: Set<String> {
        let local = Set((model.catalog?.docs ?? []).filter(\.localOnly).map(\.reference))
        return docs.subtracting(local)
    }

    var body: some View {
        VStack(spacing: 0) {
            HStack(alignment: .top, spacing: 16) {
                VStack(alignment: .leading, spacing: 5) {
                    Text(allReady ? "Ready Offline" : "Prepare for Offline Work").font(.title2.weight(.semibold))
                    Text("Select the references, starters, and service images you want to keep available. Export portable copies for another compatible Mac.")
                        .font(.callout).foregroundStyle(.secondary)
                }
                Spacer()
                Button("Import Bundle…", action: chooseImport)
                Menu("Export Selected…") {
                    Button("Documentation and Starters…", action: export).disabled(portableDocs.isEmpty && starters.isEmpty)
                    Button("Image Archives…", action: exportImages).disabled(images.isEmpty)
                }.disabled(portableDocs.isEmpty && starters.isEmpty && images.isEmpty)
            }.padding(18)
            Divider()
            if model.catalog != nil {
                HStack {
                    Text("\(docs.count) docsets · \(starters.count) starters · \(images.count) images selected").font(.caption).foregroundStyle(.secondary)
                    Spacer()
                    Toggle("Show All Versions", isOn: $showAllVersions).toggleStyle(.checkbox).font(.caption)
                }.padding(.horizontal, 18).padding(.vertical, 10)
                selectionList
            }
            Divider()
            HStack {
                if checking { ProgressView().controlSize(.small) }
                if let error { Text(error).foregroundStyle(.orange).font(.callout).textSelection(.enabled) }
                else { Text(allReady ? "Selected components passed their offline readiness checks." : "Verification is specific to this Mac. Custom project dependencies require their own offline build test.").font(.callout).foregroundStyle(.secondary) }
                Spacer()
                Button("Check Readiness") { Task { await check() } }.buttonStyle(.borderedProminent)
                    .disabled(checking || (docs.isEmpty && starters.isEmpty && images.isEmpty))
            }.padding(18)
        }
        .task(id: model.catalog != nil) {
            if !seeded, let catalog = model.catalog {
                seeded = true
                docs = Set(catalog.docs.filter { $0.installed && $0.active != false }.map(\.reference))
            }
        }
        .task {
            await infrastructure.refresh()
            images = Set(infrastructure.archived.map(\.reference))
        }
        .task(id: projectPath) {
            guard !seededProject, starters.isEmpty, let projectPath else { return }
            do {
                let project: OfflineStatus = try await Core.request(CoreRequest("offline.status", path: projectPath))
                guard starters.isEmpty, !checking else { return }
                seededProject = true
                starters = Set(project.starters.compactMap { $0.template?.reference })
            } catch is CancellationError {} catch { self.error = error.localizedDescription }
        }
        .onChange(of: readinessSelection) { _, _ in
            selectionRevision += 1
            checkedSelection = nil
            error = nil
        }
        .sheet(isPresented: Binding(get: { importPath != nil }, set: { if !$0 { importPath = nil } })) {
            if let importPath { BundleImportView(path: importPath, model: model) }
        }
    }

    private var selectionList: some View {
        ControlList {
            if let runtimes = status?.runtimes, !runtimes.isEmpty {
                ControlListSection("Project Toolchains") {
                    ForEach(runtimes) { runtime in
                        ControlListRow {
                            HStack {
                                VStack(alignment: .leading, spacing: 4) {
                                    Text(runtime.name + (runtime.version.map { " · \($0)" } ?? ""))
                                    Text(runtime.detail).font(.caption).foregroundStyle(.secondary)
                                }
                                Spacer()
                                readiness(runtime.readyOffline, detail: runtime.detail)
                            }
                        }
                    }
                }
            }
            ControlListSection("Documentation") {
                ForEach(visibleDocs, id: \.reference) { pack in
                    ControlListRow {
                        HStack {
                            Toggle(isOn: selection(pack.reference, in: $docs)) {
                                VStack(alignment: .leading, spacing: 3) {
                                    Text(pack.name)
                                    Text(displayDocVersion(pack.version)).font(.caption).foregroundStyle(.secondary)
                                    if pack.localOnly { Text("This Mac only · excluded from exports").font(.caption).foregroundStyle(.secondary) }
                                }
                            }.toggleStyle(.checkbox)
                            Spacer()
                            if let checked = status?.docs.first(where: { $0.id == pack.id && ($0.version == pack.version || $0.version == nil) }) {
                                readiness(checked.readyOffline, detail: checked.detail)
                            } else { Text(pack.installed ? "Installed" : "Not Installed").font(.caption).foregroundStyle(.secondary) }
                        }
                    }
                }
            }
            ControlListSection("Starter Dependencies and Toolchains") {
                ForEach(visibleStarters, id: \.reference) { starter in
                    ControlListRow {
                        VStack(alignment: .leading, spacing: 6) {
                            HStack {
                                Toggle(isOn: selection(starter.reference, in: $starters)) {
                                    Text("\(starter.name) · \(starter.version)")
                                }.toggleStyle(.checkbox)
                                Spacer()
                                if let result = status?.starters.first(where: { $0.template?.reference == starter.reference || $0.id == starter.reference }) {
                                    readiness(result.readyOffline, detail: result.error)
                                }
                                if starter.isInstalled {
                                    Button("Verify Offline") { Task { await model.startJob(CoreRequest("jobs.start", id: starter.reference, kind: "verify")) } }
                                        .buttonStyle(.bordered).controlSize(.small)
                                        .accessibilityLabel("Verify \(starter.name) \(starter.version) Offline")
                                }
                            }
                            if let result = status?.starters.first(where: { $0.template?.reference == starter.reference }) {
                                ForEach((result.checks ?? []).filter { !$0.ok }, id: \.name) { check in
                                    Text("\(check.name): \(check.detail)").font(.caption).foregroundStyle(.orange)
                                }
                            }
                        }
                    }
                }
            }
            ControlListSection("Infrastructure Images") {
                ForEach(infrastructure.catalog?.services ?? [], id: \.reference) { service in
                    ControlListRow {
                        HStack {
                            Toggle(isOn: selection(service.reference, in: $images)) {
                                Text("\(service.name) · \(service.version)")
                            }.toggleStyle(.checkbox)
                            Spacer()
                            if checkedSelection == readinessSelection, let checked = checkedStatus?.images.first(where: { $0.reference == service.reference }) {
                                readiness(checked.readyOffline, detail: checked.status)
                            } else {
                                let image = infrastructure.image(service.reference)
                                Text(image?.verifiedArchive == true ? "Archived" : image?.archived == true ? "Verify Archive" : "Not Archived")
                                    .font(.caption).foregroundStyle(.secondary)
                            }
                        }
                    }
                }
            }
        }
    }

    private func selection(_ reference: String, in values: Binding<Set<String>>) -> Binding<Bool> {
        Binding(get: { values.wrappedValue.contains(reference) }, set: {
            if $0 { values.wrappedValue.insert(reference) } else { values.wrappedValue.remove(reference) }
        })
    }
    private func readiness(_ ready: Bool, detail: String?) -> some View {
        Label(ready ? "Verified" : "Needs Preparation", systemImage: ready ? "checkmark.circle" : "exclamationmark.circle")
            .font(.caption).foregroundStyle(ready ? Color.green : .orange).help(detail ?? "")
    }
    private func check() async {
        guard !checking else { return }
        let selection = readinessSelection
        let revision = selectionRevision
        checking = true
        checkedSelection = nil
        error = nil
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
        panel.title = "Inspect Offline Bundle"; panel.prompt = "Inspect"
        panel.allowedContentTypes = [UTType(filenameExtension: "devlinebundle") ?? .data]
        if panel.runModal() == .OK { importPath = panel.url?.path(percentEncoded: false) }
    }
    private func export() {
        let panel = NSSavePanel()
        panel.title = "Export Offline Bundle"; panel.nameFieldStringValue = "Devline-offline.devlinebundle"
        panel.allowedContentTypes = [UTType(filenameExtension: "devlinebundle") ?? .data]
        if panel.runModal() == .OK, let path = panel.url?.path(percentEncoded: false) {
            Task { await model.startJob(CoreRequest("jobs.start", path: path, kind: "bundle_export", docs: portableDocs.sorted(), starters: starters.sorted())) }
        }
    }
    private func exportImages() {
        let panel = NSSavePanel()
        panel.title = "Export Image Archives"; panel.nameFieldStringValue = "Devline-images.devlineimages"
        if panel.runModal() == .OK, let path = panel.url?.path(percentEncoded: false) {
            Task { await model.startJob(CoreRequest("jobs.start", path: path, kind: "infra_export", images: images.sorted())) }
        }
    }
}

private struct BundleImportView: View {
    let path: String
    @ObservedObject var model: WorkspaceModel
    @Environment(\.dismiss) private var dismiss
    @State private var inspection: BundleInspection?
    @State private var error: String?
    @State private var trust = false

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text("Import Offline Bundle").font(.title2.weight(.semibold))
            Text(URL(fileURLWithPath: path).lastPathComponent).foregroundStyle(.secondary)
            if let inspection {
                Text("\(inspection.docs.count) docsets · \(inspection.starters.count) starters · \(ByteCountFormatter.string(fromByteCount: Int64(inspection.sizeBytes), countStyle: .file))")
                ScrollView {
                    VStack(alignment: .leading, spacing: 10) {
                        ForEach(inspection.docs) { Text("\($0.name) · \($0.version)") }
                        ForEach(inspection.starters) { Text("\($0.name) · \($0.version)") }
                        ForEach(inspection.problems.indices, id: \.self) { index in
                            Label("\(inspection.problems[index].id): \(inspection.problems[index].reason)", systemImage: "exclamationmark.circle").foregroundStyle(.orange)
                        }
                        ForEach((inspection.warnings ?? []).indices, id: \.self) { index in
                            Text(inspection.warnings![index].reason).foregroundStyle(.orange)
                        }
                    }.frame(maxWidth: .infinity, alignment: .leading)
                }
                if inspection.requiresTrust {
                    Toggle("I trust the source of these starter scripts", isOn: $trust)
                    Text("This bundle includes starters outside Devline’s bundled catalog. Import copies their files; verification and project commands can execute their scripts.")
                        .font(.caption).foregroundStyle(.secondary)
                }
                Text("Toolchains are not transferred. Verify starters on this Mac after importing.").font(.caption).foregroundStyle(.secondary)
            } else if error == nil { ProgressView("Checking every file and compatibility…") }
            if let error { Text(error).foregroundStyle(.orange).textSelection(.enabled) }
            HStack {
                Spacer()
                Button("Cancel") { dismiss() }.keyboardShortcut(.cancelAction)
                Button("Import") {
                    dismiss()
                    Task { await model.startJob(CoreRequest("jobs.start", path: path, kind: "bundle_import", trustStarters: trust)) }
                }.buttonStyle(.borderedProminent).disabled(inspection?.compatible != true || (inspection?.requiresTrust == true && !trust))
            }
        }.padding(22).frame(width: 620, height: 430)
            .task {
                do { inspection = try await Core.request(CoreRequest("bundle.inspect", path: path)) }
                catch { self.error = error.localizedDescription }
            }
    }
}
