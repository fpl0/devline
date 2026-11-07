import AppKit
import SwiftUI

struct ProjectReadiness: Decodable, Sendable {
    let path, status, detail: String
    let readyOffline: Bool
    let verifiedAt: Double?
}

struct ToolboxStatus: Decodable, Sendable {
    struct Dependencies: Decodable, Sendable { let state: String; let detail: String?; let verifiedAt: Double? }
    let readyOffline: Bool
    let status: String
    let library: OfflineStatus
    let infrastructure: InfrastructureStack?
    let images: [InfrastructureStatus.Image]
    let docker: InfrastructureStatus.Docker?
    let projectDependencies: Dependencies
}

struct ProjectWorkspaceView: View {
    @ObservedObject var model: WorkspaceModel
    @ObservedObject var infrastructure: InfrastructureLibrary
    let project: ProjectContext?
    let openProject, newProject, showDocs, showInfrastructure: () -> Void
    @State private var stack: InfrastructureStack?
    @State private var readiness: ToolboxStatus?
    @State private var refreshToken = UUID()
    @State private var error: String?
    @State private var checking = false
    @State private var configuring = false
    @State private var log: String?
    @State private var copied = false
    private var refreshKey: String { (project?.path ?? "") + model.jobs.filter { !$0.isActive }.map(\.id).joined() }
    private var projectReadiness: ProjectReadiness? {
        guard let project, let dependencies = readiness?.projectDependencies else { return nil }
        let ready = dependencies.state == "verified_project"
        return ProjectReadiness(path: project.path, status: ready ? "ready_offline" : "needs_preparation",
            detail: dependencies.detail ?? "Prepare and test this project for offline work.",
            readyOffline: ready, verifiedAt: dependencies.verifiedAt)
    }

    var body: some View {
        Group {
            if let project { projectBody(project) }
            else {
                ContentUnavailableView {
                    Label("Your project, ready to work", systemImage: "folder")
                } description: {
                    Text("Open a folder to see its toolchains, documentation, and local services together.")
                } actions: {
                    Button("Open Project…", action: openProject).buttonStyle(.borderedProminent)
                    Button("New Project…", action: newProject)
                }
            }
        }
        .task(id: refreshKey) { await refresh() }
        .sheet(isPresented: $configuring) {
            if let project {
                ConfigureServicesView(library: infrastructure, path: project.path, suggested: project.origin == nil ? [] : (model.catalog?.templates.first { $0.reference == project.origin?.reference }?.services ?? [])) {
                    configuring = false
                    Task { await refresh() }
                }
            }
        }
        .sheet(isPresented: Binding(get: { log != nil }, set: { if !$0 { log = nil } })) {
            VStack(alignment: .leading, spacing: 14) {
                HStack { Text("Service Logs").font(.title2.weight(.semibold)); Spacer(); Button("Done") { log = nil }.keyboardShortcut(.cancelAction) }
                ScrollView([.vertical, .horizontal]) { Text(log ?? "").font(.system(.caption, design: .monospaced)).textSelection(.enabled).frame(maxWidth: .infinity, alignment: .leading) }
            }.padding(22).frame(width: 760, height: 480)
        }
    }

    private func projectBody(_ project: ProjectContext) -> some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 26) {
                HStack(alignment: .top, spacing: 16) {
                    Image(systemName: "folder.fill").font(.system(size: 34)).foregroundStyle(Color.accentColor)
                    VStack(alignment: .leading, spacing: 7) {
                        Text(URL(fileURLWithPath: project.path).lastPathComponent).font(.largeTitle.weight(.semibold))
                        Text((project.path as NSString).abbreviatingWithTildeInPath).font(.callout).foregroundStyle(.secondary).textSelection(.enabled)
                        Text(project.stacks.isEmpty ? "Custom project" : project.stacks.map(displayStack).joined(separator: " · ")).font(.callout.weight(.medium))
                    }
                    Spacer()
                    Menu {
                        Button("Show in Finder") { NSWorkspace.shared.activateFileViewerSelecting([URL(fileURLWithPath: project.path)]) }
                        Button("Copy Terminal Command") { copy("cd \(shellQuote(project.path))") }
                        Button("Open Another Project…", action: openProject)
                    } label: { Image(systemName: "ellipsis.circle") }.accessibilityLabel("Project actions")
                }
                readinessPanel(project)
                VStack(alignment: .leading, spacing: 15) {
                    HStack {
                        Text("Local services").font(.title3.weight(.semibold))
                        Spacer()
                        if stack?.configured != true { Button("Configure Services…") { configuring = true } }
                        Button { Task { await refresh() } } label: { Image(systemName: "arrow.clockwise") }.help("Refresh project status").accessibilityLabel("Refresh project status")
                    }
                    if let stack, stack.configured {
                        HStack(spacing: 10) {
                            Button(stack.running == true ? "Start / Reconcile" : "Start Services") { job("infra_stack_up", project.path) }
                                .buttonStyle(.borderedProminent).disabled(infrastructure.status?.docker.available != true)
                            Button("Stop Services") { job("infra_stack_down", project.path) }.disabled(infrastructure.status?.docker.available != true)
                            Button("View Logs") { Task { await loadLogs(project.path) } }.disabled(infrastructure.status?.docker.available != true)
                            Spacer()
                            Text(stack.statusError != nil ? "Status Unavailable" : stack.healthy == true ? "All Services Healthy" : stack.running == true ? "Starting or Needs Attention" : "Stopped").font(.caption).foregroundStyle(.secondary)
                        }
                        if let statusError = stack.statusError {
                            Label(statusError, systemImage: "exclamationmark.circle").font(.callout).foregroundStyle(.orange).textSelection(.enabled)
                        }
                        ForEach(stack.services) { service in serviceRow(service) }
                        HStack {
                            Text("Stopping keeps your project’s data volumes.").font(.caption).foregroundStyle(.secondary)
                            Spacer()
                            Button(copied ? "Copied" : "Copy Credentials") { Task { await copyCredentials(project.path) } }.controlSize(.small)
                        }
                    } else {
                        VStack(alignment: .leading, spacing: 10) {
                            Label("Add the services your app needs", systemImage: "server.rack").font(.headline)
                            Text("Choose databases, queues, storage, search, and authentication. Each project gets pinned images, local ports, credentials, and its own data volumes.")
                                .font(.callout).foregroundStyle(.secondary)
                            Button("Choose Services…") { configuring = true }
                        }.padding(20).frame(maxWidth: .infinity, alignment: .leading).background(.quaternary.opacity(0.35), in: RoundedRectangle(cornerRadius: 12))
                    }
                }
                Divider()
                HStack(alignment: .top, spacing: 20) {
                    VStack(alignment: .leading, spacing: 8) {
                        Label("Project references", systemImage: "book.closed").font(.headline)
                        Text("\(project.dependencies?.count ?? 0) detected dependencies. Search documentation in the context of this project and pin the versions you use.").font(.callout).foregroundStyle(.secondary)
                        Button("Browse Project Docs", action: showDocs)
                    }.frame(maxWidth: .infinity, alignment: .leading)
                    VStack(alignment: .leading, spacing: 8) {
                        Label("Image library", systemImage: "archivebox").font(.headline)
                        Text("Prepare or restore service images independently of Docker’s cache. Keep a portable copy for another compatible Mac.").font(.callout).foregroundStyle(.secondary)
                        Button("Manage Infrastructure", action: showInfrastructure)
                    }.frame(maxWidth: .infinity, alignment: .leading)
                }
                if let error { Label(error, systemImage: "exclamationmark.circle").font(.callout).foregroundStyle(.orange).textSelection(.enabled) }
            }.padding(30).frame(maxWidth: 980, alignment: .leading).frame(maxWidth: .infinity, alignment: .topLeading)
        }
    }

    private func readinessPanel(_ project: ProjectContext) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Label(projectReadiness?.readyOffline == true ? "Build verified offline" : "Prepare this project for offline work", systemImage: projectReadiness?.readyOffline == true ? "checkmark.shield.fill" : "checkmark.shield")
                    .font(.headline)
                Spacer()
                if checking { ProgressView().controlSize(.small) }
                Button(projectReadiness?.readyOffline == true ? "Verify Again" : "Prepare Offline") {
                    Task { await model.startJob(CoreRequest("jobs.start", path: project.path, kind: "project_prepare", offline: projectReadiness?.readyOffline == true)) }
                }.buttonStyle(.borderedProminent).disabled(checking || model.jobs.contains { $0.isActive && $0.kind == "project_prepare" })
            }
            if let readiness {
                Text(projectReadiness?.detail ?? readiness.projectDependencies.detail ?? "Devline prepares local dependencies, then tests the project with external networking denied.")
                    .font(.callout).foregroundStyle(.secondary).fixedSize(horizontal: false, vertical: true)
                HStack(spacing: 20) {
                    readinessItem("Documentation", ready: readiness.library.docs.allSatisfy(\.readyOffline))
                    readinessItem("Project Dependencies", ready: projectReadiness?.readyOffline == true)
                    if !readiness.images.isEmpty { readinessItem("Image Archives", ready: readiness.images.allSatisfy(\.verifiedArchive)) }
                }.font(.caption)
                if let gap = project.docsets?.first(where: { $0.id == "swiftui" && $0.version == nil && $0.availableVersions.isEmpty }) {
                    Text(gap.detail).font(.callout).foregroundStyle(.orange)
                        .fixedSize(horizontal: false, vertical: true)
                    HStack {
                        Button("Prepare SwiftUI Reference") {
                            Task { await model.startJob(CoreRequest("jobs.start", id: "swiftui", kind: "docs_prepare")) }
                        }.disabled(model.jobs.contains { $0.isActive && $0.kind == "docs_prepare" })
                        Link("Apple’s Website", destination: URL(string: "https://developer.apple.com/documentation/swiftui/")!)
                    }.font(.callout)
                }
                if !readiness.images.isEmpty, readiness.docker?.available != true {
                    Text("Start your local Docker engine to run this project’s services.").font(.caption).foregroundStyle(.orange)
                }
            } else { Text("Checks use the project’s pinned references and configured services.").font(.callout).foregroundStyle(.secondary) }
        }.padding(18).background(.quaternary.opacity(0.35), in: RoundedRectangle(cornerRadius: 12))
    }
    private func readinessItem(_ title: String, ready: Bool) -> some View {
        Label(title, systemImage: ready ? "checkmark.circle.fill" : "circle.dashed").foregroundStyle(ready ? Color.secondary : .orange)
    }
    private func serviceRow(_ service: InfrastructureStack.Service) -> some View {
        let catalog = infrastructure.catalog?.services.first { $0.reference == service.reference }
        let image = infrastructure.image(service.reference)
        return HStack(alignment: .center, spacing: 14) {
            Image(systemName: catalog?.symbol ?? "server.rack").font(.title3).foregroundStyle(.secondary).frame(width: 26)
            VStack(alignment: .leading, spacing: 5) {
                Text("\(catalog?.name ?? service.id) · \(service.version)").font(.callout.weight(.medium))
                HStack(spacing: 12) {
                    ForEach(service.ports) { port in
                        Button { copy("127.0.0.1:\(port.hostPort)") } label: { Text("\(port.name) :\(String(port.hostPort))").font(.caption.monospacedDigit()) }
                            .buttonStyle(.link).help("Copy local endpoint")
                    }
                }
            }
            Spacer()
            if image?.verifiedArchive == true { Label("Archived", systemImage: "archivebox").font(.caption).foregroundStyle(.secondary) }
            else if image?.archived == true {
                Button("Verify Archive") { Task { await model.startJob(CoreRequest("jobs.start", id: service.reference, kind: "infra_verify")) } }.controlSize(.small)
            }
            else { Button("Prepare Image") { Task { await model.startJob(CoreRequest("jobs.start", id: service.reference, kind: "infra_prepare")) } }.controlSize(.small) }
        }.padding(15).background(.background, in: RoundedRectangle(cornerRadius: 10)).overlay { RoundedRectangle(cornerRadius: 10).stroke(.quaternary, lineWidth: 1) }
    }

    private func refresh(verify: Bool = false) async {
        guard let project else { return }
        let path = project.path
        let token = UUID()
        refreshToken = token
        checking = true
        error = nil
        defer { if refreshToken == token { checking = false } }
        do {
            async let current: InfrastructureStack = Core.request(CoreRequest("infra.stack.status", path: path))
            async let checked: ToolboxStatus = Core.request(CoreRequest(verify ? "toolbox.check" : "toolbox.status", path: path))
            let result = try await (current, checked)
            guard !Task.isCancelled, refreshToken == token else { return }
            stack = result.0; readiness = result.1
            await infrastructure.refresh()
        } catch is CancellationError {}
        catch { if refreshToken == token { self.error = error.localizedDescription } }
    }
    private func job(_ kind: String, _ path: String) { Task { await model.startJob(CoreRequest("jobs.start", path: path, kind: kind)) } }
    private func loadLogs(_ path: String) async {
        do {
            let result: InfrastructureStack = try await Core.request(CoreRequest("infra.stack.logs", path: path, tail: 120))
            log = result.log ?? "No log output yet."
        } catch { self.error = error.localizedDescription }
    }
    private func copy(_ text: String) { NSPasteboard.general.clearContents(); copied = NSPasteboard.general.setString(text, forType: .string) }
    private func copyCredentials(_ path: String) async {
        struct Credentials: Decodable { let credentials: [String: String] }
        do {
            let result: Credentials = try await Core.request(CoreRequest("infra.stack.credentials", path: path))
            copy(result.credentials.sorted { $0.key < $1.key }.map { "\($0.key)=\(shellQuote($0.value))" }.joined(separator: "\n"))
        } catch { self.error = error.localizedDescription }
    }
}

struct ConfigureServicesView: View {
    @Environment(\.dismiss) private var dismiss
    @ObservedObject var library: InfrastructureLibrary
    let path: String
    let suggested: [String]
    let completion: () -> Void
    @State private var selected: Set<String> = []
    @State private var ports: [String: Int] = [:]
    @State private var saving = false
    @State private var error: String?
    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text("Configure Local Services").font(.title2.weight(.semibold))
            Text("Choose what this project needs. Images can be prepared now or later; starting services uses only your saved archives.")
                .font(.callout).foregroundStyle(.secondary)
            ServiceSelectionView(library: library, selected: $selected, ports: $ports).frame(height: 340)
            if let error { Text(error).font(.callout).foregroundStyle(.red).textSelection(.enabled) }
            HStack {
                Text("Ports bind to 127.0.0.1.").font(.caption).foregroundStyle(.secondary)
                Spacer()
                if saving { ProgressView().controlSize(.small) }
                Button("Cancel") { dismiss() }.keyboardShortcut(.cancelAction).disabled(saving)
                Button("Configure") { Task { await save() } }.buttonStyle(.borderedProminent).keyboardShortcut(.defaultAction).disabled(selected.isEmpty || saving)
            }
        }.padding(24).frame(width: 620).interactiveDismissDisabled(saving)
            .task { selected = Set(suggested); await library.refresh() }
    }
    private func save() async {
        saving = true; error = nil
        defer { saving = false }
        do {
            let _: InfrastructureStack = try await Core.request(CoreRequest("infra.stack.create", path: path, services: selected.sorted(), ports: ports))
            completion()
        } catch { self.error = error.localizedDescription }
    }
}

struct ServiceSelectionView: View {
    @ObservedObject var library: InfrastructureLibrary
    @Binding var selected: Set<String>
    @Binding var ports: [String: Int]
    var required: Set<String> = []
    var body: some View {
        List(library.catalog?.services ?? [], id: \.reference) { service in
            VStack(alignment: .leading, spacing: 8) {
                Toggle(isOn: Binding(get: { selected.contains(service.reference) }, set: { select(service, enabled: $0) })) {
                    HStack {
                        Label(service.name, systemImage: service.symbol)
                        Spacer()
                        Text(service.version).font(.caption).foregroundStyle(.secondary)
                        if required.contains(service.reference) { Text("Required").font(.caption).foregroundStyle(.secondary) }
                        Text(imageStatus(service)).font(.caption).foregroundStyle(.secondary)
                    }
                }.toggleStyle(.checkbox)
                    .disabled(required.contains(service.reference) || required.contains { $0.hasPrefix(service.id + "@") && $0 != service.reference })
                if selected.contains(service.reference) {
                    ForEach(service.ports) { port in
                        HStack {
                            Text("\(port.name) port").font(.caption).foregroundStyle(.secondary)
                            Spacer()
                            TextField("Port", value: Binding(get: { ports["\(service.id):\(port.id)"] ?? port.hostPort }, set: { ports["\(service.id):\(port.id)"] = $0 }), format: .number.grouping(.never))
                                .textFieldStyle(.roundedBorder).frame(width: 90).accessibilityLabel("\(service.name) \(port.name) host port")
                        }.padding(.leading, 22)
                    }
                }
            }.padding(.vertical, 5)
        }.listStyle(.inset).overlay { if library.loading && library.catalog == nil { ProgressView() } }
    }
    private func select(_ service: InfrastructureService, enabled: Bool) {
        let portPrefix = service.id + ":"
        if enabled {
            selected = selected.filter { !$0.hasPrefix(service.id + "@") }
            selected.insert(service.reference)
            let availablePorts = Set(service.ports.map { portPrefix + $0.id })
            ports = ports.filter { !$0.key.hasPrefix(portPrefix) || availablePorts.contains($0.key) }
        } else {
            selected.remove(service.reference)
            ports = ports.filter { !$0.key.hasPrefix(portPrefix) }
        }
    }
    private func imageStatus(_ service: InfrastructureService) -> String {
        if let image = library.image(service.reference), image.archived {
            return image.verifiedArchive ? "Archived" : "Verify archive"
        }
        return service.image.compressedBytes.map(storageSize) ?? "Download needed"
    }
}
