import AppKit
import SwiftUI

struct ProjectWorkspaceView: View {
    @ObservedObject var model: WorkspaceModel
    @ObservedObject var infrastructure: InfrastructureLibrary
    let project: ProjectContext?
    let openProject, newProject, showDocs, showInfrastructure: () -> Void
    let showStarter: (String) -> Void
    let showToolchains, closeProject: () -> Void
    @State private var stack: InfrastructureStack?
    @State private var readiness: ToolboxStatus?
    @State private var refreshToken = UUID()
    @State private var error: String?
    @State private var checking = false
    @State private var preparing = false
    @State private var configuring = false
    @State private var log: String?
    @State private var copied = false
    @State private var helpRequest: ProjectHelpTarget?
    private var refreshKey: [String] { [project?.path ?? ""] + model.completedJobIDs }
    private var projectJob: JobRecord? {
        model.jobs.first { $0.targetPath == project?.path && ["toolbox_prepare", "project_prepare"].contains($0.kind) }
    }
    private var serviceJob: JobRecord? {
        model.jobs.first { $0.targetPath == project?.path && ["infra_stack_up", "infra_stack_down", "infra_stack_add", "service_clients_add"].contains($0.kind) }
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
                ConfigureServicesView(model: model, library: infrastructure, path: project.path, runtime: project.stack, existing: stack,
                    suggested: project.origin == nil ? [] : (model.catalog?.templates.first { $0.reference == project.origin?.reference }?.services ?? [])) {
                    configuring = false
                }
            }
        }
        .sheet(item: $helpRequest) { DevlineHelpView(id: $0.id) }
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
                        if let origin = project.origin {
                            Button { showStarter(origin.reference) } label: {
                                Text("Created from \(model.catalog?.templates.first { $0.reference == origin.reference }?.name ?? origin.starter) · \(origin.version)")
                            }.buttonStyle(.link).font(.caption).help("Open the exact starter release used for this project")
                        }
                    }
                    Spacer()
                    Menu {
                        Button("Show in Finder") { NSWorkspace.shared.activateFileViewerSelecting([URL(fileURLWithPath: project.path)]) }
                        Button("Copy Terminal Command") { copy("cd \(shellQuote(project.path))") }
                        Button("Open Another Project…", action: openProject)
                        Button("Close Project", action: closeProject)
                    } label: { Image(systemName: "ellipsis.circle") }.accessibilityLabel("Project actions")
                }
                ProjectRunView(path: project.path, preparationRevision: refreshKey, refreshServices: { Task { await refresh() } })
                readinessPanel(project)
                VStack(alignment: .leading, spacing: 15) {
                    HStack {
                        Text("Local services").font(.title3.weight(.semibold))
                        Spacer()
                        Button(stack?.configured == true ? "Add Services…" : "Choose Services…") { configuring = true }
                            .disabled(checking || serviceJob?.isActive == true)
                        Button { Task { await refresh() } } label: { Image(systemName: "arrow.clockwise") }.help("Refresh project status").accessibilityLabel("Refresh project status")
                    }
                    if let job = serviceJob, job.isActive || job.needsAttention {
                        HStack(spacing: 10) {
                            if job.isActive { ProgressView().controlSize(.small) }
                            else { Image(systemName: job.icon).foregroundStyle(.orange) }
                            Text(job.isActive ? job.stage : job.error ?? job.stateLabel).font(.callout).foregroundStyle(.secondary).lineLimit(2)
                            Spacer()
                            if job.isActive {
                                Button("Cancel") { Task { await cancel(job) } }.controlSize(.small)
                            }
                            Button("View Activity") { model.inspectedJob = job }.controlSize(.small)
                        }
                    }
                    if let stack, stack.configured {
                        HStack(spacing: 10) {
                            Button(stack.running == true ? "Start / Reconcile" : "Start Services") { job("infra_stack_up", project.path) }
                                .buttonStyle(.borderedProminent).disabled(infrastructure.status?.docker.available != true || serviceJob?.isActive == true)
                            Button("Stop Services") { job("infra_stack_down", project.path) }.disabled(infrastructure.status?.docker.available != true || serviceJob?.isActive == true || stack.running != true)
                            Button("View Logs") { Task { await loadLogs(project.path) } }.disabled(infrastructure.status?.docker.available != true)
                            Spacer()
                            Text(stack.statusError != nil ? "Status Unavailable" : stack.healthy == true ? "All Services Healthy" : stack.running == true ? "Starting or Needs Attention" : "Stopped").font(.caption).foregroundStyle(.secondary)
                        }
                        if let statusError = stack.statusError {
                            Label(statusError, systemImage: "exclamationmark.circle").font(.callout).foregroundStyle(.orange).textSelection(.enabled)
                        }
                        ForEach(stack.services) { service in serviceRow(service) }
                        HStack {
                            Text("Run Offline supplies these connection settings to your app. Stopping services keeps their data.").font(.caption).foregroundStyle(.secondary)
                            Spacer()
                            Button(copied ? "Copied" : "Copy Credentials") { Task { await copyCredentials(project.path) } }.controlSize(.small)
                        }
                    } else {
                        VStack(alignment: .leading, spacing: 10) {
                            Label("Add the services your app needs", systemImage: "server.rack").font(.headline)
                            Text("Choose databases, queues, storage, search, and authentication. Each project gets pinned images, local ports, credentials, and its own data volumes.")
                                .font(.callout).foregroundStyle(.secondary)
                            Button("Choose Services…") { configuring = true }
                                .disabled(checking || serviceJob?.isActive == true)
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
        VStack(alignment: .leading, spacing: 14) {
            HStack {
                Label(readiness?.readyOffline == true ? "Ready to work offline" : "Prepare this project for offline work",
                      systemImage: readiness?.readyOffline == true ? "checkmark.shield.fill" : "checkmark.shield")
                    .font(.headline)
                Spacer()
                if checking || preparing { ProgressView().controlSize(.small) }
                Button(readiness?.readyOffline == true ? "Verify Again" : projectJob?.needsAttention == true ? "Retry Preparation" : "Prepare Offline") {
                    preparing = true
                    Task {
                        defer { preparing = false }
                        await model.startJob(CoreRequest("jobs.start", path: project.path, kind: "toolbox_prepare", offline: readiness?.readyOffline == true), inspect: false)
                    }
                }.buttonStyle(.borderedProminent).disabled(checking || preparing || projectJob?.isActive == true)
            }
            if let job = projectJob, job.isActive || job.needsAttention || job.state == "cancelled" {
                HStack(spacing: 12) {
                    if job.isActive { ProgressView().controlSize(.small) }
                    else { Image(systemName: job.icon).foregroundStyle(.orange) }
                    VStack(alignment: .leading, spacing: 4) {
                        Text(job.isActive ? job.stage : job.stateLabel).font(.callout.weight(.medium))
                        if let failure = job.error { Text(failure).font(.caption).foregroundStyle(.secondary).lineLimit(2) }
                        else if !job.isActive { Text("Prepared items are kept. Review the remaining steps and try again.").font(.caption).foregroundStyle(.secondary) }
                    }
                    Spacer()
                    Button("View Activity") { model.inspectedJob = job }.controlSize(.small)
                }
            }
            if let readiness {
                Text(readiness.readyOffline ? "The project’s dependencies, references, toolchain, and selected services are ready on this Mac." : "One preparation brings together the project’s dependencies, matching documentation, and pinned service images.")
                    .font(.callout).foregroundStyle(.secondary).fixedSize(horizontal: false, vertical: true)
                VStack(alignment: .leading, spacing: 10) {
                    let runtimes = readiness.library.runtimes ?? []
                    readinessRow("Toolchain", ready: !runtimes.isEmpty && runtimes.allSatisfy(\.readyOffline),
                        detail: runtimes.isEmpty ? "No supported toolchain detected. Devline only identifies tools installed on this Mac." : runtimes.map(\.detail).joined(separator: " · "), action: "View Toolchains", perform: showToolchains)
                    readinessRow("Dependencies", ready: readiness.projectDependencies.verified,
                        detail: readiness.projectDependencies.detail ?? "Prepare and test this project for offline work.")
                    let missing = readiness.library.docs.filter { !$0.readyOffline }
                    readinessRow("Documentation", ready: missing.isEmpty,
                        detail: missing.isEmpty ? "\(readiness.library.docs.count) matching references available locally." : missing.map { "\($0.id): \($0.detail ?? "Needs preparation")" }.joined(separator: " · "), action: "Browse Project Docs", perform: showDocs)
                    if !readiness.images.isEmpty {
                        readinessRow("Service images", ready: readiness.images.allSatisfy(\.verifiedArchive),
                            detail: "\(readiness.images.filter(\.verifiedArchive).count) of \(readiness.images.count) pinned image archives verified.", action: "Manage Images", perform: showInfrastructure)
                        readinessRow("Docker", ready: readiness.docker?.available == true,
                            detail: readiness.docker?.available == true ? "Local engine available for your project services." : "Start your local Docker engine to run this project’s services.")
                    }
                }
            } else { Text("Checking the project’s dependencies, references, toolchain, and services…").font(.callout).foregroundStyle(.secondary) }
        }.padding(18).background(.quaternary.opacity(0.35), in: RoundedRectangle(cornerRadius: 12))
    }

    private func readinessRow(_ title: String, ready: Bool, detail: String, action: String? = nil, perform: (() -> Void)? = nil) -> some View {
        HStack(alignment: .top, spacing: 10) {
            Image(systemName: ready ? "checkmark.circle.fill" : "circle.dashed")
                .foregroundStyle(ready ? Color.secondary : .orange).frame(width: 18)
            VStack(alignment: .leading, spacing: 3) {
                Text(title).font(.callout.weight(.medium))
                Text(detail).font(.caption).foregroundStyle(.secondary).fixedSize(horizontal: false, vertical: true)
            }
            Spacer(minLength: 8)
            if let action, let perform { Button(action, action: perform).controlSize(.small) }
        }
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
            if let help = catalog?.helpRef {
                Button("Setup Guide") { helpRequest = ProjectHelpTarget(id: help) }.controlSize(.small)
            }
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
    private func job(_ kind: String, _ path: String) { Task { await model.startJob(CoreRequest("jobs.start", path: path, kind: kind), inspect: false) } }
    private func cancel(_ job: JobRecord) async {
        do { let _: Acknowledgment = try await Core.request(CoreRequest("jobs.cancel", jobId: job.id)); await model.refreshJobs() }
        catch { self.error = error.localizedDescription }
    }
    private func loadLogs(_ path: String) async {
        do {
            let result: InfrastructureStack = try await Core.request(CoreRequest("infra.stack.logs", path: path, tail: 120))
            log = result.log ?? "No log output yet."
        } catch { self.error = error.localizedDescription }
    }
    private func copy(_ text: String) { copied = copyToPasteboard(text) }
    private func copyCredentials(_ path: String) async {
        struct Credentials: Decodable { let credentials: [String: String]; let environment: [String: String]? }
        do {
            let result: Credentials = try await Core.request(CoreRequest("infra.stack.credentials", path: path))
            copy((result.environment ?? result.credentials).sorted { $0.key < $1.key }.map { "export \($0.key)=\(shellQuote($0.value))" }.joined(separator: "\n"))
        } catch { self.error = error.localizedDescription }
    }
}

private struct ProjectHelpTarget: Identifiable { let id: String }
