import AppKit
import SwiftUI

private enum Workspace: String, CaseIterable, Identifiable {
    case projects = "Projects"
    case docs = "Docs"
    case services = "Services"
    case library = "Offline Library"

    var id: String { rawValue }
    var icon: String {
        switch self {
        case .projects: "folder"
        case .services: "server.rack"
        case .docs: "book.closed"
        case .library: "tray.full"
        }
    }

    static func restore(_ saved: String) -> Self {
        if let current = Self(rawValue: saved) { return current }
        switch saved {
        case "Infrastructure": return .services
        case "Library", "Ready Offline", "Runtimes", "Updates", "Activity": return .library
        default: return .projects
        }
    }
}

private enum LibrarySection: String, CaseIterable {
    case assets = "Assets"
    case toolchains = "Toolchains"
    case updates = "Updates"
}

struct WorkspaceView: View {
    @Environment(\.scenePhase) private var scenePhase
    @Environment(\.openWindow) private var openWindow
    @ObservedObject var model: WorkspaceModel
    @StateObject private var reading = ReadingSession()
    @StateObject private var infrastructure = InfrastructureLibrary()
    @State private var workspace: Workspace? = Workspace.restore(UserDefaults.standard.string(forKey: "devline.workspace") ?? "")
    @State private var showingStarters = UserDefaults.standard.string(forKey: "devline.workspace") == "Templates"
    @AppStorage("devline.librarySection") private var librarySection: LibrarySection = {
        switch UserDefaults.standard.string(forKey: "devline.workspace") {
        case "Runtimes": return .toolchains
        case "Updates": return .updates
        default: return .assets
        }
    }()
    @State private var showingActivity = UserDefaults.standard.string(forKey: "devline.workspace") == "Activity"
    @State private var showingManageDocs = false
    @State private var manageDocsForInstall = false
    @State private var manageDocsPackID: String?
    @State private var showingProjectServices = true
    @AppStorage("devline.focusReading") private var focusReading = false
    @State private var columnVisibility: NavigationSplitViewVisibility = .all
    @State private var restoredSession = false
    @State private var query = ""
    @State private var docStack: String?
    @State private var docPack: String?
    @State private var directDocumentID: String?
    @State private var projectContext: ProjectContext?
    @State private var openingProject = false
    @State private var projectRequest: UUID?
    @State private var navigationRequest: UUID?
    @State private var focusRequest = 0
    @State private var lookupRevision = 0
    @State private var projectScopeRequested = false
    @State private var starterReference: String?
    @State private var newStarter: Starter?
    @State private var showingAppUpdates = false

    var body: some View {
        NavigationSplitView(columnVisibility: $columnVisibility) {
            List(selection: $workspace) {
                ForEach(Workspace.allCases) { page in
                    Label(page.rawValue, systemImage: page.icon).tag(page)
                }
            }
            .listStyle(.sidebar)
            .navigationSplitViewColumnWidth(min: 165, ideal: 190, max: 245)
        } detail: {
            Group {
                if model.catalog == nil {
                    CatalogLoadingView(model: model)
                } else {
                    workspaceContent
                }
            }
            .navigationTitle((workspace ?? .projects).rawValue)
            .safeAreaInset(edge: .bottom, spacing: 0) {
                if let operation = model.operation {
                    StatusBar(text: operation, working: true)
                } else if let notice = model.notice {
                    StatusBar(text: notice, working: false, dismiss: { model.notice = nil })
                }
            }
            .toolbar {
                if model.jobs.contains(where: \.isActive) {
                    ToolbarItem(placement: .primaryAction) {
                        Button { showingActivity = true } label: {
                            Label("Activity", systemImage: "clock.arrow.circlepath")
                        }.help("View progress and logs")
                    }
                }
                if workspace == .projects && projectContext != nil && !showingStarters {
                    ToolbarItem(placement: .primaryAction) {
                        Button(action: beginNewProject) {
                            Label("New from Starter", systemImage: "plus")
                        }.help("New from Starter (⌘N)")
                    }
                }
            }
        }
        .frame(minWidth: 840, minHeight: 530)
        .task {
            NativeLookup.shared.revealWorkspace = { openWindow(id: "workspace") }
            guard !restoredSession else { return }
            if navigationRequest == nil,
               let path = UserDefaults.standard.string(forKey: "devline.projectPath"),
               FileManager.default.fileExists(atPath: path) {
                if await loadProjectContext(path) { model.rememberProject(path) }
            }
            guard !Task.isCancelled else { return }
            restoredSession = true
            if navigationRequest == nil { reading.restoreSession() }
            columnVisibility = focusReading && workspace == .docs ? .detailOnly : .all
        }
        .task(id: scenePhase) {
            if scenePhase == .active {
                await model.refresh()
                await reading.refresh()
                if restoredSession, !Task.isCancelled, let path = projectContext?.path {
                    await loadProjectContext(path)
                }
            } else {
                reading.persist()
            }
        }
        .onReceive(NotificationCenter.default.publisher(for: NSApplication.willTerminateNotification)) { _ in reading.persist() }
        .onReceive(NotificationCenter.default.publisher(for: .devlineLookup)) { notification in
            if let request = notification.object as? LookupRequest { lookUp(request) }
        }
        .sheet(item: $newStarter) { starter in
            NewProjectView(model: model, starter: starter, infrastructure: infrastructure) { path in
                newStarter = nil
                openProject(path)
            }
        }
        .sheet(item: $model.inspectedJob) { job in JobDetailsView(job: job, model: model) }
        .sheet(isPresented: $showingAppUpdates) { AppUpdatesSheet() }
        .sheet(isPresented: $showingManageDocs) {
            ManageDocsView(model: model, installRequested: manageDocsForInstall, requestedPackID: manageDocsPackID) { pack in
                showingManageDocs = false
                browseDocset(pack)
            }
        }
        .sheet(isPresented: $showingActivity) { ActivitySheet(model: model) }
        .alert("Devline", isPresented: Binding(
            get: { model.error != nil },
            set: { if !$0 { model.error = nil } }
        )) {
            Button("OK", role: .cancel) { model.error = nil }
        } message: {
            Text(model.error ?? "")
        }
        .onReceive(NotificationCenter.default.publisher(for: .devlineFind)) { _ in
            beginNavigation()
            workspace = .docs
            focusRequest += 1
        }
        .onReceive(NotificationCenter.default.publisher(for: .devlineNew)) { _ in beginNewProject() }
        .onReceive(NotificationCenter.default.publisher(for: .devlineOpenProject)) { _ in chooseProjectFolder() }
        .onReceive(NotificationCenter.default.publisher(for: .devlineOpenRecent)) { notification in
            if let path = notification.object as? String { openProject(path) }
        }
        .onReceive(NotificationCenter.default.publisher(for: .devlineLibrary)) { _ in openLibrary(.assets) }
        .onReceive(NotificationCenter.default.publisher(for: .devlineUpdates)) { _ in openLibrary(.updates) }
        .onReceive(NotificationCenter.default.publisher(for: .devlineActivity)) { _ in showingActivity = true }
        .onReceive(NotificationCenter.default.publisher(for: .devlineAppUpdates)) { _ in showingAppUpdates = true }
        .onChange(of: workspace) { _, page in
            UserDefaults.standard.set(page?.rawValue ?? Workspace.projects.rawValue, forKey: "devline.workspace")
            columnVisibility = focusReading && page == .docs ? .detailOnly : .all
        }
        .onChange(of: focusReading) { _, focused in columnVisibility = focused && workspace == .docs ? .detailOnly : .all }
        .onChange(of: model.libraryRevision) { _, _ in
            if let path = projectContext?.path { Task { await loadProjectContext(path) } }
            Task { await infrastructure.refresh() }
        }
        .onOpenURL(perform: handleURL)
    }

    @ViewBuilder
    private var workspaceContent: some View {
        switch workspace ?? .projects {
        case .projects:
            if showingStarters {
                VStack(spacing: 0) {
                    HStack {
                        Button { showingStarters = false } label: {
                            Label("Back to Projects", systemImage: "chevron.left")
                        }.buttonStyle(.borderless)
                        Spacer()
                    }.padding(.horizontal, 20).padding(.vertical, 12)
                    Divider()
                    TemplatesView(model: model, selection: $starterReference, newStarter: $newStarter, openDocs: browseDocset)
                }
            } else {
                ProjectWorkspaceView(model: model, project: projectContext,
                    openProject: chooseProjectFolder, newProject: beginNewProject, showDocs: browseProjectDocs,
                    showInfrastructure: { openServices(project: false) }, openRecent: openProject, showStarter: showStarter,
                    showToolchains: { openLibrary(.toolchains) }, showProjectServices: { openServices(project: true) },
                    closeProject: clearProject)
                    .id(projectContext?.path)
            }
        case .services:
            VStack(spacing: 0) {
                if projectContext != nil {
                    Picker("Services Section", selection: $showingProjectServices) {
                        Text("Project Services").tag(true)
                        Text("Image Library").tag(false)
                    }.pickerStyle(.segmented).labelsHidden().fixedSize()
                        .frame(maxWidth: .infinity).padding(12)
                    Divider()
                }
                if let projectContext, showingProjectServices {
                    ProjectServicesView(model: model, infrastructure: infrastructure, project: projectContext)
                        .id(projectContext.path)
                } else {
                    InfrastructureView(model: model, library: infrastructure, openDocs: openInfrastructureReference,
                                       openProjectServices: projectContext == nil ? nil : { openServices(project: true) })
                }
            }
        case .docs:
            DocsView(
                model: model,
                reading: reading,
                query: $query,
                stack: $docStack,
                packReference: $docPack,
                directDocumentID: $directDocumentID,
                projectContext: projectContext,
                updateProject: { context in
                    guard !openingProject, projectContext?.path == context.path else { return }
                    projectRequest = nil
                    projectContext = context
                },
                clearProject: clearProject,
                focusRequest: focusRequest,
                lookupRevision: lookupRevision,
                projectScopeRequested: projectScopeRequested,
                openLibrary: { openDocsManager() },
                installDocset: { openDocsManager(install: true, packID: $0) }
            )
        case .library:
            VStack(spacing: 0) {
                Picker("Offline Library Section", selection: $librarySection) {
                    ForEach(LibrarySection.allCases, id: \.self) { Text($0.rawValue).tag($0) }
                }
                .pickerStyle(.segmented).labelsHidden().fixedSize()
                .frame(maxWidth: .infinity).padding(12)
                Divider()
                libraryContent
            }
        }
    }

    @ViewBuilder private var libraryContent: some View {
        switch librarySection {
        case .assets:
            if let catalog = model.catalog {
                LibraryView(model: model, infrastructure: infrastructure, catalog: catalog, projectPath: projectContext?.path,
                            manageDocs: { openDocsManager() })
            }
        case .toolchains:
            RuntimesView(
                model: model,
                projectStack: projectContext?.stack,
                showStarter: showStarter,
                showDocs: { stack in
                    projectScopeRequested = false
                    directDocumentID = nil
                    docStack = stack
                    docPack = nil
                    query = ""
                    workspace = .docs
                    focusRequest += 1
                },
                showLibrary: { openLibrary(.assets) }
            )
        case .updates:
            UpdatesView(model: model)
        }
    }

    private func openLibrary(_ section: LibrarySection) {
        librarySection = section
        workspace = .library
    }

    private func openServices(project: Bool) {
        showingProjectServices = project
        workspace = .services
    }

    private func openDocsManager(install: Bool = false, packID: String? = nil) {
        manageDocsForInstall = install
        manageDocsPackID = packID
        showingManageDocs = true
    }

    private func handleURL(_ url: URL) {
        if let request = LookupRequest(url: url) {
            lookUp(request)
            return
        }
        let scheme = url.scheme ?? ""
        guard scheme == "devline" || scheme == "devkit" else { return }
        guard url.host == "docs" else { return }
        guard let components = URLComponents(url: url, resolvingAgainstBaseURL: false) else { return }
        let items: [URLQueryItem] = components.queryItems ?? []
        guard let id = items.first(where: { $0.name == "id" })?.value, !id.isEmpty else { return }
        let anchor: String? = items.first(where: { $0.name == "anchor" })?.value
        let focus: String? = items.first(where: { $0.name == "focus" })?.value
        let location = ReferenceLocation(id: id, anchor: anchor, focusText: focus)
        beginNavigation()
        projectScopeRequested = false
        reading.open(location)
        directDocumentID = id
        workspace = .docs
    }

    private func openInfrastructureReference(_ reference: String) {
        Task {
            let release = reference.split(separator: "/", maxSplits: 1).first.map(String.init)
            guard let pack = model.catalog?.docs.first(where: { $0.reference == release }) else {
                model.error = "The official offline reference is unavailable for this release."
                return
            }
            if !pack.installed {
                guard await model.mutate([CoreRequest("docs.install", id: pack.id, version: pack.version)],
                    label: "Preparing infrastructure reference", completion: "Infrastructure reference available offline.") else { return }
            }
            beginNavigation()
            projectScopeRequested = false
            reading.open(ReferenceLocation(id: reference))
            directDocumentID = reference
            workspace = .docs
        }
    }

    private func beginNewProject() {
        if workspace == .projects && showingStarters,
           let starter = model.catalog?.templates.first(where: { $0.reference == starterReference && $0.isInstalled }) {
            newStarter = starter
        } else {
            workspace = .projects
            showingStarters = true
        }
    }

    private func chooseProjectFolder() {
        guard !openingProject, newStarter == nil else { return }
        let panel = NSOpenPanel()
        panel.title = "Open Project Folder"
        panel.prompt = "Open"
        panel.canChooseDirectories = true
        panel.canChooseFiles = false
        panel.allowsMultipleSelection = false
        if let path = projectContext?.path {
            panel.directoryURL = URL(fileURLWithPath: path, isDirectory: true)
        }
        if panel.runModal() == .OK, let url = panel.url {
            openProject(url.path(percentEncoded: false))
        }
    }

    @discardableResult
    private func beginNavigation() -> UUID {
        let token = UUID()
        navigationRequest = token
        openingProject = false
        reading.request = nil
        return token
    }

    @discardableResult
    private func loadProjectContext(_ path: String, navigation: UUID? = nil) async -> Bool {
        if let navigation {
            guard navigationRequest == navigation else { return false }
        } else {
            guard projectRequest == nil, !openingProject, projectContext == nil || projectContext?.path == path else { return false }
        }
        let request = UUID()
        projectRequest = request
        defer { if projectRequest == request { projectRequest = nil } }
        do {
            let context: ProjectContext = try await Core.request(CoreRequest("project.context", path: path))
            guard projectRequest == request, navigation == nil || navigationRequest == navigation else { return false }
            projectContext = context
            UserDefaults.standard.set(context.path, forKey: "devline.projectPath")
            return true
        } catch is CancellationError {
            return false
        } catch {
            if projectRequest == request, navigation == nil || navigationRequest == navigation { model.error = error.localizedDescription }
            return false
        }
    }

    private func openProject(_ path: String) {
        let navigation = beginNavigation()
        openingProject = true
        Task {
            defer { if navigationRequest == navigation { openingProject = false } }
            guard await loadProjectContext(path, navigation: navigation), navigationRequest == navigation else { return }
            docStack = projectContext?.stack.flatMap { stack in
                model.catalog?.docs.contains { $0.installed && $0.stack == stack } == true ? stack : nil
            }
            directDocumentID = nil
            docPack = nil
            query = ""
            showingStarters = false
            workspace = .projects
            guard let resolvedPath = projectContext?.path else { return }
            model.rememberProject(resolvedPath)
            do {
                let readiness: ToolboxStatus = try await Core.request(CoreRequest("toolbox.status", path: resolvedPath))
                guard navigationRequest == navigation, projectContext?.path == resolvedPath else { return }
                if !readiness.readyOffline {
                    guard !model.jobs.contains(where: { $0.isActive && $0.targetPath == resolvedPath && ["toolbox_prepare", "project_prepare"].contains($0.kind) }) else { return }
                    await model.startJob(CoreRequest("jobs.start", path: resolvedPath, kind: "toolbox_prepare"), inspect: false)
                }
            } catch is CancellationError {}
            catch { model.error = error.localizedDescription }
        }
    }

    private func clearProject() {
        beginNavigation()
        projectRequest = nil
        projectContext = nil
        projectScopeRequested = false
        UserDefaults.standard.removeObject(forKey: "devline.projectPath")
        docStack = nil
        docPack = nil
    }

    private func browseProjectDocs() {
        beginNavigation()
        projectScopeRequested = true
        query = ""
        directDocumentID = nil
        docPack = nil
        docStack = projectContext?.stack
        lookupRevision += 1
        focusReading = false
        workspace = .docs
        focusRequest += 1
    }

    private func browseDocset(_ pack: DocPack) {
        guard pack.installed else {
            model.error = "Prepare \(pack.name) \(displayDocVersion(pack.version)) before browsing it."
            return
        }
        beginNavigation()
        projectScopeRequested = false
        query = ""
        directDocumentID = nil
        docStack = pack.stack
        docPack = pack.reference
        lookupRevision += 1
        focusReading = false
        workspace = .docs
    }

    private func showStarter(_ reference: String) {
        guard model.catalog?.templates.contains(where: { $0.reference == reference }) == true else {
            model.error = "The original starter release (\(reference)) is no longer in this library. Your project keeps its own files and dependencies."
            return
        }
        starterReference = reference
        showingStarters = true
        workspace = .projects
    }

    private func lookUp(_ request: LookupRequest) {
        let navigation = beginNavigation()
        projectScopeRequested = false
        if let path = request.path {
            openingProject = true
            Task {
                defer { if navigationRequest == navigation { openingProject = false } }
                guard await loadProjectContext(path, navigation: navigation), navigationRequest == navigation else { return }
                projectScopeRequested = request.stack == nil
                docStack = request.stack ?? projectContext?.stack
                finishLookup(query: request.query, message: request.message)
            }
        } else {
            docStack = request.stack ?? projectContext?.stack
            finishLookup(query: request.query, message: request.message)
        }
    }

    private func finishLookup(query: String, message: String? = nil) {
        directDocumentID = nil
        docPack = nil
        self.query = query
        lookupRevision += 1
        model.notice = message
        focusReading = false
        workspace = .docs
        focusRequest += 1
    }
}

private struct CatalogLoadingView: View {
    @ObservedObject var model: WorkspaceModel

    var body: some View {
        VStack(spacing: 16) {
            if model.loadingCatalog || model.catalogError == nil {
                ProgressView()
                Text("Opening your library…").font(.headline)
            } else {
                Image(systemName: "externaldrive.badge.exclamationmark").font(.largeTitle).foregroundStyle(.secondary)
                Text("Couldn’t Open the Library").font(.title2.weight(.semibold))
                Text(model.catalogError ?? "Please try again.")
                    .font(.callout).foregroundStyle(.secondary).multilineTextAlignment(.center)
                    .textSelection(.enabled).frame(maxWidth: 440)
                Button("Retry") { Task { await model.refresh() } }
                    .buttonStyle(.borderedProminent)
            }
        }
        .padding(32)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

private struct StatusBar: View {
    let text: String
    let working: Bool
    var dismiss: (() -> Void)?

    var body: some View {
        HStack(spacing: 9) {
            if working { ProgressView().controlSize(.small) }
            else { Image(systemName: "checkmark.circle").foregroundStyle(.green) }
            Text(text).font(.callout)
            Spacer()
            if let dismiss {
                Button(action: dismiss) { Image(systemName: "xmark") }
                    .buttonStyle(.borderless)
                    .help("Dismiss status")
                    .accessibilityLabel("Dismiss status")
            }
        }
        .padding(.horizontal, 18)
        .padding(.vertical, 9)
        .frame(maxWidth: .infinity)
        .background(.bar)
        .overlay(alignment: .top) { Divider() }
    }
}
