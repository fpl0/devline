import AppKit
import SwiftUI

struct ProjectWorkspaceView: View {
    @ObservedObject var model: WorkspaceModel
    let project: ProjectContext?
    let openProject, newProject, showDocs, showInfrastructure: () -> Void
    let openRecent: (String) -> Void
    let showStarter: (String) -> Void
    let showToolchains, showProjectServices, closeProject: () -> Void
    @State private var readiness: ToolboxStatus?
    @State private var refreshToken = UUID()
    @State private var error: String?
    @State private var checking = false
    @State private var preparing = false
    private var refreshKey: [String] { [project?.path ?? ""] + model.completedJobIDs }
    private var projectJob: JobRecord? {
        model.jobs.first { $0.targetPath == project?.path && ["toolbox_prepare", "project_prepare"].contains($0.kind) }
    }

    var body: some View {
        Group {
            if let project { projectBody(project) }
            else { projectsHome }
        }
        .task(id: refreshKey) { await refresh() }
    }

    private var projectsHome: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 24) {
                VStack(alignment: .leading, spacing: 10) {
                    Text("Open or create a project").font(.title.weight(.semibold)).accessibilityAddTraits(.isHeader)
                    HStack(spacing: 10) {
                        Button("Open Folder…", action: openProject).buttonStyle(.borderedProminent)
                        Button("New from Starter…", action: newProject)
                    }
                }
                if !model.recentProjects.isEmpty {
                    VStack(alignment: .leading, spacing: 0) {
                        Text("Recent Projects").font(.headline).padding(.bottom, 8)
                        ForEach(model.recentProjects) { recent in
                            Divider()
                            Button { openRecent(recent.path) } label: {
                                HStack(spacing: 12) {
                                    Image(systemName: "folder").foregroundStyle(.secondary)
                                    VStack(alignment: .leading, spacing: 3) {
                                        Text(recent.name).font(.callout.weight(.medium))
                                        Text((recent.path as NSString).abbreviatingWithTildeInPath)
                                            .font(.caption).foregroundStyle(.secondary)
                                            .lineLimit(1).truncationMode(.middle)
                                    }
                                    Spacer()
                                    Image(systemName: "chevron.right").font(.caption).foregroundStyle(.tertiary)
                                }.padding(.vertical, 10).contentShape(Rectangle())
                            }
                            .buttonStyle(.plain)
                            .contextMenu { Button("Remove from Recent Projects") { model.forgetProject(recent.path) } }
                        }
                    }
                }
            }.padding(28).frame(maxWidth: 720, alignment: .leading).frame(maxWidth: .infinity, alignment: .leading)
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
                        Button("Copy Terminal Command") { _ = copyToPasteboard("cd \(shellQuote(project.path))") }
                        Button("Open Folder…", action: openProject)
                        if model.recentProjects.contains(where: { $0.path != project.path }) {
                            Menu("Open Recent") {
                                ForEach(model.recentProjects.filter { $0.path != project.path }) { recent in
                                    Button(recent.name) { openRecent(recent.path) }.help(recent.path)
                                }
                            }
                        }
                        Button("Close Project", action: closeProject)
                    } label: { Image(systemName: "ellipsis.circle") }.accessibilityLabel("Project actions")
                }
                ProjectRunView(path: project.path, preparationRevision: refreshKey, refreshServices: { Task { await refresh() } })
                readinessPanel(project)
                Divider()
                HStack(alignment: .top, spacing: 20) {
                    VStack(alignment: .leading, spacing: 8) {
                        Label("Project references", systemImage: "book.closed").font(.headline)
                        Text("\(project.dependencies?.count ?? 0) detected dependencies. Search documentation in the context of this project and pin the versions you use.").font(.callout).foregroundStyle(.secondary)
                        Button("Browse Project Docs", action: showDocs)
                    }.frame(maxWidth: .infinity, alignment: .leading)
                    VStack(alignment: .leading, spacing: 8) {
                        Label("Local services", systemImage: "server.rack").font(.headline)
                        Text("Choose, start, or stop services for this project.").font(.callout).foregroundStyle(.secondary)
                        Button("Manage Project Services", action: showProjectServices)
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
    private func refresh() async {
        guard let project else { return }
        let token = UUID()
        refreshToken = token
        checking = true
        error = nil
        defer { if refreshToken == token { checking = false } }
        do {
            let checked: ToolboxStatus = try await Core.request(CoreRequest("toolbox.status", path: project.path))
            guard !Task.isCancelled, refreshToken == token else { return }
            readiness = checked
        } catch is CancellationError {}
        catch { if refreshToken == token { self.error = error.localizedDescription } }
    }
}
