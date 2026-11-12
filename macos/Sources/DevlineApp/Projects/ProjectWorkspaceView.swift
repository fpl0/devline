import AppKit
import SwiftUI

/// The last readiness read for each project. A page shows it at once and refreshes behind it.
@MainActor private var knownReadiness: [String: ToolboxStatus] = [:]

/// Reads a project's readiness ahead of time, so its page is complete on the first visit too.
@MainActor func prefetchProjectReadiness(_ path: String) async {
    guard knownReadiness[path] == nil,
          let status: ToolboxStatus = try? await Core.request(CoreRequest("toolbox.status", path: path), priority: .utility)
    else { return }
    if knownReadiness[path] == nil { knownReadiness[path] = status }
}

struct ProjectWorkspaceView: View {
    @PageObservedObject var model: WorkspaceModel
    @ObservedObject var infrastructure: InfrastructureLibrary
    let project: ProjectContext
    let showDocs: () -> Void
    let showDocset: (String) -> Void
    let showInfrastructure: () -> Void
    let showStarter: (String) -> Void
    let showToolchains, showProjectServices, closeProject: () -> Void
    let online: Bool
    var visible = true
    @State private var fresh: ToolboxStatus?
    private var readiness: ToolboxStatus? { fresh ?? knownReadiness[project.path] }
    @State private var refreshToken = UUID()
    @State private var error: String?
    @State private var checking = false
    @State private var preparing = false
    @State private var editors: [URL] = []
    @State private var addingPackages = false
    @Preference("devline.editor") private var preferredEditor = ""

    private var refreshKey: [String] { [project.path] + model.completedJobIDs }
    private var projectJob: JobRecord? {
        model.jobs.first { $0.targetPath == project.path && ["toolbox_prepare", "project_prepare"].contains($0.kind) }
    }
    private var name: String { URL(fileURLWithPath: project.path).lastPathComponent }
    private var folder: URL { URL(fileURLWithPath: project.path, isDirectory: true) }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 22) {
                header
                section("Run", symbol: "play.circle") {
                    ProjectRunView(path: project.path, preparationRevision: refreshKey, visible: visible,
                                   refreshServices: { Task { await refresh() } })
                }
                readinessPanel
                if let error {
                    Label(error, systemImage: "exclamationmark.circle").font(.callout).foregroundStyle(.orange).textSelection(.enabled)
                }
            }
            .padding(.horizontal, 32).padding(.vertical, 28)
            .frame(maxWidth: 960, alignment: .leading)
            .frame(maxWidth: .infinity)
        }
        .task(id: refreshKey) { await refresh() }
        .task { await loadEditors() }
        .onPageShown(visible) { await refresh() }
        .sheet(isPresented: $addingPackages) {
            AddPackagesView(model: model, project: project) { Task { await refresh() } }
        }
    }

    // MARK: Header

    private var header: some View {
        HStack(alignment: .top, spacing: 16) {
            Image(systemName: "folder.fill").font(.system(size: 32)).foregroundStyle(Color.accentColor)
                .frame(width: 42).accessibilityHidden(true)
            VStack(alignment: .leading, spacing: 6) {
                HStack(alignment: .firstTextBaseline, spacing: 12) {
                    Text(name).font(.largeTitle.weight(.semibold)).lineLimit(1).accessibilityAddTraits(.isHeader)
                    statusPill
                }
                Text((project.path as NSString).abbreviatingWithTildeInPath)
                    .font(.callout).foregroundStyle(.secondary).lineLimit(1).truncationMode(.middle)
                    .textSelection(.enabled).help(project.path)
                HStack(spacing: 6) {
                    Text(project.stacks.isEmpty ? "Custom project" : project.stacks.map(displayStack).joined(separator: " · "))
                        .font(.callout.weight(.medium))
                    if let origin = project.origin {
                        Text("·").foregroundStyle(.tertiary)
                        Button { showStarter(origin.reference) } label: {
                            Text("From \(model.catalog?.templates.first { $0.reference == origin.reference }?.name ?? origin.starter) \(origin.version)")
                        }.buttonStyle(.link).font(.callout).help("Show the starter this project was created from")
                    }
                }
            }
            Spacer(minLength: 12)
            editorButton
            Menu {
                Button("Show in Finder") { NSWorkspace.shared.activateFileViewerSelecting([folder]) }
                Button("Open in Terminal", action: openTerminal)
                Button("Copy Path") { _ = copyToPasteboard(project.path) }
                Divider()
                Button("Close Project", action: closeProject)
            } label: { Image(systemName: "ellipsis.circle") }
                .menuIndicator(.hidden).fixedSize().accessibilityLabel("Project actions")
        }
    }

    @ViewBuilder private var statusPill: some View {
        let active = projectJob?.isActive == true
        if active || readiness != nil {
            let ready = readiness?.readyOffline == true
            HStack(spacing: 5) {
                if active { ProgressView().controlSize(.mini) }
                else { Image(systemName: ready ? "checkmark.circle.fill" : "exclamationmark.circle.fill") }
                Text(active ? "Preparing" : ready ? "Ready offline" : "Not ready offline")
            }
            .font(.caption.weight(.semibold))
            .foregroundStyle(active ? Color.secondary : ready ? .green : .orange)
            .padding(.horizontal, 9).padding(.vertical, 4)
            .background((active ? Color.secondary : ready ? .green : .orange).opacity(0.14), in: Capsule())
            .accessibilityElement(children: .combine)
        }
    }

    @ViewBuilder private var editorButton: some View {
        if let editor = editors.first(where: { $0.path == preferredEditor }) ?? editors.first {
            Menu {
                ForEach(editors, id: \.path) { editor in
                    Button(editorName(editor)) { open(in: editor) }
                }
                Divider()
                Button("Terminal", action: openTerminal)
                Button("Finder") { NSWorkspace.shared.open(folder) }
            } label: {
                Label("Open in \(editorName(editor))", systemImage: "chevron.left.forwardslash.chevron.right")
            } primaryAction: {
                open(in: editor)
            }.fixedSize()
        } else {
            Button("Open Folder") { NSWorkspace.shared.open(folder) }
        }
    }

    // MARK: Readiness

    private var readinessPanel: some View {
        let ready = readiness?.readyOffline == true
        let jobActive = projectJob?.isActive == true
        return section("Offline readiness", symbol: "checkmark.shield") {
            VStack(alignment: .leading, spacing: 14) {
                HStack(alignment: .top, spacing: 12) {
                    Text(readiness == nil ? "Checking what this project needs…"
                         : ready ? "Everything this project needs is on this Mac and was tested with the network blocked."
                         : online ? "Something this project needs isn’t on this Mac yet. Prepare while you’re connected and Devline fetches and tests it."
                         : "Something this project needs isn’t on this Mac, and you’re offline. You can still check it against what is here.")
                        .font(.callout).foregroundStyle(.secondary).fixedSize(horizontal: false, vertical: true)
                    Spacer(minLength: 12)
                    if preparing || (checking && readiness == nil) { ProgressView().controlSize(.small) }
                    if ready {
                        Button("Check Again") { prepare(offline: true) }.disabled(preparing || jobActive)
                            .help("Re-run the offline test with what is on this Mac")
                    } else {
                        Button(!online ? "Check With What’s Here" : projectJob?.needsAttention == true ? "Try Again" : "Prepare for Offline") { prepare(offline: !online) }
                            .buttonStyle(.borderedProminent).disabled(readiness == nil || preparing || jobActive)
                    }
                }
                if let job = projectJob, job.isActive || job.needsAttention || job.state == "cancelled" {
                    HStack(spacing: 10) {
                        if job.isActive { ProgressView().controlSize(.small) }
                        else { Image(systemName: job.icon).foregroundStyle(.orange) }
                        VStack(alignment: .leading, spacing: 3) {
                            Text(job.isActive ? job.stage : job.stateLabel).font(.callout.weight(.medium))
                            if let failure = job.error { Text(failure).font(.caption).foregroundStyle(.secondary).lineLimit(2) }
                            else if !job.isActive { Text("What was prepared is kept. Fix what remains below and try again.").font(.caption).foregroundStyle(.secondary) }
                        }
                        Spacer()
                        Button("Details") { model.inspectedJob = job }.controlSize(.small)
                    }
                    .padding(10).background(Color.primary.opacity(0.05), in: RoundedRectangle(cornerRadius: 8))
                }
                if let readiness {
                    VStack(alignment: .leading, spacing: 0) {
                        toolchainRow(readiness)
                        Divider().padding(.leading, 28)
                        row("Dependencies", ready: readiness.projectDependencies.verified,
                            detail: readiness.projectDependencies.detail ?? "Not prepared and tested yet.",
                            action: project.stack == nil ? nil : "Add Packages…", perform: { addingPackages = true })
                        Divider().padding(.leading, 28)
                        docsRow(readiness)
                        Divider().padding(.leading, 28)
                        servicesRow(readiness)
                    }
                }
            }
        }
    }

    private func toolchainRow(_ readiness: ToolboxStatus) -> some View {
        let runtimes = readiness.library.runtimes ?? []
        let home = FileManager.default.homeDirectoryForCurrentUser.path
        let detail = runtimes.isEmpty
            ? "No supported toolchain found. Devline uses the compilers and interpreters already installed on this Mac."
            : runtimes.map { $0.detail.replacingOccurrences(of: home, with: "~") }.joined(separator: "\n")
        return row("Toolchain", ready: !runtimes.isEmpty && runtimes.allSatisfy(\.readyOffline), detail: detail,
                   action: "Toolchains", perform: showToolchains)
    }

    private func docsRow(_ readiness: ToolboxStatus) -> some View {
        let docs = readiness.library.docs
        let missing = docs.filter { !$0.readyOffline }
        let available = docs.filter(\.readyOffline)
        return rowFrame(ready: missing.isEmpty) {
            VStack(alignment: .leading, spacing: 6) {
                Text("Documentation").font(.callout.weight(.medium))
                if docs.isEmpty {
                    Text("No docsets match this project’s dependencies. Search still covers everything installed.")
                        .font(.caption).foregroundStyle(.secondary)
                }
                if !available.isEmpty {
                    FlowRow(spacing: 6) {
                        ForEach(available) { doc in
                            Button { showDocset(doc.reference) } label: {
                                Label(docName(doc), systemImage: "book.closed").font(.caption)
                            }
                            .controlSize(.small)
                            .help(project.docsets?.first { $0.id == doc.id }?.detail ?? "Read offline")
                        }
                    }
                }
                ForEach(missing) { doc in
                    Text("\(docTitle(doc.id)): \(doc.detail ?? "Not installed yet.")")
                        .font(.caption).foregroundStyle(.orange).fixedSize(horizontal: false, vertical: true)
                }
            }
        } trailing: {
            Button("Search Project Docs", action: showDocs).controlSize(.small)
        }
    }

    private func servicesRow(_ readiness: ToolboxStatus) -> some View {
        let images = readiness.images
        let saved = images.filter(\.verifiedArchive).count
        let docker = readiness.docker?.available == true
        let names = images.map { image in
            infrastructure.catalog?.services.first { $0.reference == image.reference }?.name ?? image.id.capitalized
        }
        let detail = images.isEmpty
            ? "This project runs without local services. Add a database, queue, or cache when you need one."
            : "\(names.joined(separator: ", ")) · \(saved == images.count ? "images saved" : "\(saved) of \(images.count) images saved")"
                + (docker ? "" : "\nStart Docker to run them. Saved images stay available while it is stopped.")
        return row("Services", ready: images.isEmpty || (saved == images.count && docker), detail: detail,
                   action: images.isEmpty ? "Add Services" : "Manage", perform: showProjectServices)
    }

    private func row(_ title: String, ready: Bool, detail: String, action: String? = nil, perform: (() -> Void)? = nil) -> some View {
        rowFrame(ready: ready) {
            VStack(alignment: .leading, spacing: 3) {
                Text(title).font(.callout.weight(.medium))
                Text(detail).font(.caption).foregroundStyle(.secondary).fixedSize(horizontal: false, vertical: true)
                    .textSelection(.enabled)
            }
        } trailing: {
            if let action, let perform { Button(action, action: perform).controlSize(.small) }
        }
    }

    private func rowFrame(ready: Bool, @ViewBuilder content: () -> some View, @ViewBuilder trailing: () -> some View) -> some View {
        HStack(alignment: .top, spacing: 10) {
            Image(systemName: ready ? "checkmark.circle.fill" : "exclamationmark.circle.fill")
                .foregroundStyle(ready ? Color.green : .orange).frame(width: 18)
                .accessibilityLabel(ready ? "Ready" : "Needs attention")
            content()
            Spacer(minLength: 8)
            trailing()
        }.padding(.vertical, 10)
    }

    private func section(_ title: String, symbol: String, @ViewBuilder content: () -> some View) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            Label(title, systemImage: symbol).font(.headline).accessibilityAddTraits(.isHeader)
            content()
        }
        .padding(18)
        .frame(maxWidth: .infinity, alignment: .leading)
        .surface(radius: 12)
    }

    private func docTitle(_ id: String) -> String {
        model.catalog?.docs.first { $0.id == id }?.name ?? id
    }

    private func docName(_ doc: OfflineStatus.Doc) -> String {
        let version = doc.version.map { displayDocVersion($0).components(separatedBy: " · ").first ?? $0 }
        return [docTitle(doc.id), version].compactMap { $0 }.joined(separator: " ")
    }

    // MARK: Actions

    private func prepare(offline: Bool) {
        preparing = true
        Task {
            defer { preparing = false }
            await model.startJob(CoreRequest("jobs.start", path: project.path, kind: "toolbox_prepare", offline: offline), inspect: false)
        }
    }

    private func refresh() async {
        let token = UUID()
        refreshToken = token
        if !checking { checking = true }
        if error != nil { error = nil }
        defer { if refreshToken == token { checking = false } }
        await infrastructure.loadIfNeeded(after: model.completedInfrastructureJobIDs)
        do {
            let checked: ToolboxStatus = try await Core.request(CoreRequest("toolbox.status", path: project.path))
            guard !Task.isCancelled, refreshToken == token else { return }
            fresh = checked
            knownReadiness[project.path] = checked
        } catch is CancellationError {}
        catch { if refreshToken == token { self.error = error.localizedDescription } }
    }

    private func editorName(_ editor: URL) -> String { editor.deletingPathExtension().lastPathComponent }

    private func open(in editor: URL) {
        preferredEditor = editor.path
        NSWorkspace.shared.open([folder], withApplicationAt: editor, configuration: NSWorkspace.OpenConfiguration()) { _, failure in
            if let failure { Task { @MainActor in error = failure.localizedDescription } }
        }
    }

    private func openTerminal() {
        guard let terminal = NSWorkspace.shared.urlForApplication(withBundleIdentifier: "com.apple.Terminal") else {
            _ = copyToPasteboard("cd \(shellQuote(project.path))")
            return
        }
        NSWorkspace.shared.open([folder], withApplicationAt: terminal, configuration: NSWorkspace.OpenConfiguration()) { _, failure in
            if let failure { Task { @MainActor in error = failure.localizedDescription } }
        }
    }

    private func loadEditors() async {
        let discovered = await Task.detached {
            ["com.microsoft.VSCode", "com.todesktop.230313mzl4w4u92", "dev.zed.Zed", "com.apple.dt.Xcode",
             "com.jetbrains.intellij", "com.jetbrains.rider", "com.jetbrains.pycharm", "com.jetbrains.goland",
             "com.jetbrains.rustrover", "com.jetbrains.WebStorm", "com.jetbrains.rubymine", "com.sublimetext.4", "com.panic.Nova"]
                .compactMap { NSWorkspace.shared.urlForApplication(withBundleIdentifier: $0) }
        }.value
        if !Task.isCancelled, editors != discovered { editors = discovered }
    }
}

/// Lays children out left to right and wraps onto new lines when they run out of width.
struct FlowRow: Layout {
    var spacing: CGFloat = 6

    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let rows = arrange(subviews, width: proposal.width ?? .infinity)
        return CGSize(width: proposal.width ?? rows.map(\.width).max() ?? 0,
                      height: rows.last.map { $0.y + $0.height } ?? 0)
    }

    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        for row in arrange(subviews, width: bounds.width) {
            var x = bounds.minX
            for index in row.items {
                let size = subviews[index].sizeThatFits(.unspecified)
                subviews[index].place(at: CGPoint(x: x, y: bounds.minY + row.y), proposal: ProposedViewSize(size))
                x += size.width + spacing
            }
        }
    }

    private struct Row { var items: [Int] = []; var width: CGFloat = 0; var y: CGFloat = 0; var height: CGFloat = 0 }

    private func arrange(_ subviews: Subviews, width: CGFloat) -> [Row] {
        var rows: [Row] = []
        var current = Row()
        for index in subviews.indices {
            let size = subviews[index].sizeThatFits(.unspecified)
            let extended = current.items.isEmpty ? size.width : current.width + spacing + size.width
            if !current.items.isEmpty && extended > width {
                rows.append(current)
                current = Row(y: current.y + current.height + spacing)
                current.items = [index]; current.width = size.width; current.height = size.height
            } else {
                current.items.append(index); current.width = extended; current.height = max(current.height, size.height)
            }
        }
        if !current.items.isEmpty { rows.append(current) }
        return rows
    }
}
