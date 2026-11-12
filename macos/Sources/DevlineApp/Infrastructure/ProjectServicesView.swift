import AppKit
import SwiftUI

/// The last service stack read for each project, shown at once while a fresh read runs.
@MainActor private var knownStacks: [String: InfrastructureStack] = [:]

struct ProjectServicesView: View {
    @PageObservedObject var model: WorkspaceModel
    @ObservedObject var infrastructure: InfrastructureLibrary
    let project: ProjectContext
    var visible = true
    @State private var fresh: InfrastructureStack?
    private var stack: InfrastructureStack? { fresh ?? knownStacks[project.path] }
    @State private var refreshToken = UUID()
    @State private var error: String?
    @State private var checking = false
    @State private var configuring = false
    @State private var log: String?
    @State private var copied = false
    @State private var helpRequest: ProjectHelpTarget?
    private var serviceJob: JobRecord? {
        model.jobs.first { $0.targetPath == project.path && ["infra_stack_up", "infra_stack_down", "infra_stack_add", "service_clients_add"].contains($0.kind) }
    }


    var body: some View {
        let completedJobs = model.completedInfrastructureJobIDs
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                VStack(alignment: .leading, spacing: 15) {
                    HStack {
                        Text("Services for \(URL(fileURLWithPath: project.path).lastPathComponent)").font(.title3.weight(.semibold))
                        Spacer()
                        if stack?.configured == true {
                            Button("Add Services…") { configuring = true }.disabled(checking || serviceJob?.isActive == true)
                        }
                        Button { Task { await refresh(forceLibrary: true) } } label: { Image(systemName: "arrow.clockwise") }
                            .help("Refresh project status").accessibilityLabel("Refresh project status").disabled(checking)
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
                            let dockerAvailable = stack.docker?.available == true
                            Button(stack.running == true ? "Start / Reconcile" : "Start Services") { job("infra_stack_up", project.path) }
                                .buttonStyle(.borderedProminent).disabled(!dockerAvailable || serviceJob?.isActive == true)
                            Button("Stop Services") { job("infra_stack_down", project.path) }.disabled(!dockerAvailable || serviceJob?.isActive == true || stack.running != true)
                            Button("View Logs") { Task { await loadLogs(project.path) } }.disabled(!dockerAvailable)
                            Spacer()
                            Text(stack.statusError != nil ? "Status Unavailable" : stack.healthy == true ? "All Services Healthy" : stack.running == true ? "Starting or Needs Attention" : "Stopped").font(.caption).foregroundStyle(.secondary)
                        }
                        if let statusError = stack.statusError {
                            Label(statusError, systemImage: "exclamationmark.circle").font(.callout).foregroundStyle(.orange).textSelection(.enabled)
                        }
                        ForEach(stack.services) { service in serviceRow(service) }
                        HStack {
                            Text("Run supplies these connection settings to your app. Stopping services keeps their data.").font(.caption).foregroundStyle(.secondary)
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
                        }.padding(20).frame(maxWidth: .infinity, alignment: .leading).surface(radius: 12)
                    }
                }
                if let error {
                    Label(error, systemImage: "exclamationmark.circle").font(.callout)
                        .foregroundStyle(.orange).textSelection(.enabled)
                }
            }
            .padding(.horizontal, 32).padding(.vertical, 28)
            .frame(maxWidth: 960, alignment: .leading)
            .frame(maxWidth: .infinity)
        }
        .task(id: completedJobs) { await refresh() }
        .onPageShown(visible) { await refresh(quiet: true) }
        .sheet(isPresented: $configuring) {
            ConfigureServicesView(model: model, library: infrastructure, path: project.path, runtime: project.stack, existing: stack,
                suggested: model.catalog?.templates.first { $0.reference == project.origin?.reference }?.services ?? []) {
                configuring = false
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

    /// Ports that serve a page for people, not only an API for the app.
    private static func hasWebPage(service: String, port: String) -> Bool {
        ["web", "management", "monitor"].contains(port) || (service == "keycloak" && port == "http")
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
                        HStack(spacing: 4) {
                            Button { copy("127.0.0.1:\(port.hostPort)") } label: { Text("\(port.name) :\(String(port.hostPort))").font(.caption.monospacedDigit()) }
                                .buttonStyle(.link).help("Copy local endpoint")
                            if Self.hasWebPage(service: service.id, port: port.id),
                               let url = URL(string: "http://127.0.0.1:\(port.hostPort)") {
                                Button { NSWorkspace.shared.open(url) } label: {
                                    Image(systemName: "arrow.up.forward.square").font(.caption)
                                }
                                .buttonStyle(.borderless).disabled(stack?.running != true)
                                .help(stack?.running == true ? "Open \(port.name) in your browser" : "Start the services to open \(port.name)")
                                .accessibilityLabel("Open \(port.name)")
                            }
                        }
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

    /// `quiet` brings a page that is already showing its services up to date without dimming it.
    private func refresh(forceLibrary: Bool = false, quiet: Bool = false) async {
        let token = UUID()
        refreshToken = token
        if !quiet || stack == nil { checking = true }
        if error != nil { error = nil }
        async let imageSnapshot: Void = refreshLibrary(force: forceLibrary)
        do {
            defer { if refreshToken == token { checking = false } }
            let current: InfrastructureStack = try await Core.request(CoreRequest("infra.stack.status", path: project.path))
            guard !Task.isCancelled, refreshToken == token else { return }
            fresh = current
            knownStacks[project.path] = current
        } catch is CancellationError {}
        catch { if refreshToken == token { self.error = error.localizedDescription } }
        await imageSnapshot
    }

    private func refreshLibrary(force: Bool) async {
        if force { await infrastructure.refresh() }
        else { await infrastructure.loadIfNeeded(after: model.completedInfrastructureJobIDs) }
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
