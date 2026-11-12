import AppKit
import SwiftUI

private enum Workspace: String, Identifiable {
    case home = "Home"
    case project = "Project"
    case docs = "Docs"
    case services = "Services"
    case library = "Library"

    /// The pages that are always in the sidebar. An open project is listed with the other projects.
    static let pages: [Self] = [.home, .docs, .services, .library]

    var id: String { rawValue }
    var icon: String {
        switch self {
        case .home: "house"
        case .project: "folder"
        case .services: "server.rack"
        case .docs: "book.closed"
        case .library: "tray.full"
        }
    }

    static func restore(_ saved: String) -> Self {
        if let current = Self(rawValue: saved) { return current }
        switch saved {
        case "Projects": return .project
        case "Infrastructure": return .services
        case "Offline Library", "Templates", "Ready Offline", "Runtimes", "Updates", "Activity": return .library
        default: return .home
        }
    }
}

private enum SidebarItem: Hashable {
    case page(Workspace)
    case project(String)
}

private enum LibrarySection: String, CaseIterable {
    case assets = "Assets"
    case starters = "Starters"
    case packages = "Packages"
    case toolchains = "Toolchains"
    case updates = "Updates"

    var title: String { self == .assets ? "Contents" : rawValue }
}

struct WorkspaceView: View {
    @Environment(\.scenePhase) private var scenePhase
    @Environment(\.openWindow) private var openWindow
    @ObservedObject var model: WorkspaceModel
    @StateObject private var reading = ReadingSession()
    @StateObject private var infrastructure = InfrastructureLibrary()
    @StateObject private var runtimeInventory = RuntimeInventory()
    @StateObject private var shelves = PackageShelves()
    @StateObject private var connectivity = Connectivity()
    @State private var toolchainSelection: String?
    @State private var workspace: Workspace? = Workspace.restore(UserDefaults.standard.string(forKey: "devline.workspace") ?? "")
    /// Pages that have been built. They stay loaded, so going back to one costs nothing.
    @State private var mounted: Set<String> = []
    /// Project pages that stay loaded, most recently opened last.
    @State private var projectPages: [String] = []
    @State private var librarySection: LibrarySection = {
        if let saved = UserDefaults.standard.string(forKey: "devline.librarySection").flatMap(LibrarySection.init) { return saved }
        switch UserDefaults.standard.string(forKey: "devline.workspace") {
        case "Templates": return .starters
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
    @Preference("devline.focusReading") private var focusReading = false
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
    @State private var startersRequest = 0
    @State private var pendingProject: String?
    /// What was last read about each project, so returning to one shows it at once.
    @State private var knownProjects: [String: ProjectContext] = [:]

    private var showingAssets: Bool { workspace == .library && librarySection == .assets }

    var body: some View {
        NavigationSplitView(columnVisibility: $columnVisibility) {
            List(selection: sidebarSelection) {
                ForEach(Workspace.pages) { page in
                    Label(page.rawValue, systemImage: page.icon).tag(SidebarItem.page(page))
                }
                Section {
                    ForEach(RecentProject.listed(model.recentProjects)) { recent in
                        Label(recent.name, systemImage: "folder").tag(SidebarItem.project(recent.path))
                            .help((recent.path as NSString).abbreviatingWithTildeInPath)
                            .contextMenu {
                                Button("Show in Finder") {
                                    NSWorkspace.shared.activateFileViewerSelecting([URL(fileURLWithPath: recent.path)])
                                }
                                Button("Remove from List") { forgetProject(recent.path) }
                            }
                    }
                    if model.recentProjects.isEmpty {
                        Text("No projects yet").font(.callout).foregroundStyle(.tertiary)
                    }
                } header: {
                    HStack {
                        Text("Projects")
                        Spacer()
                        Menu {
                            Button("New Project…", action: beginNewProject)
                            Button("Open Folder…", action: chooseProjectFolder)
                        } label: {
                            Image(systemName: "plus")
                        }
                        .menuStyle(.borderlessButton).menuIndicator(.hidden).fixedSize()
                        .help("New project or open a folder").accessibilityLabel("Add Project")
                    }
                }
            }
            .listStyle(.sidebar)
            .safeAreaInset(edge: .bottom, spacing: 0) { sidebarStatus }
            .navigationSplitViewColumnWidth(min: 180, ideal: 210, max: 280)
        } detail: {
            Group {
                if model.catalog == nil {
                    CatalogLoadingView(model: model)
                } else {
                    pages
                }
            }
            .navigationTitle(pageTitle)
            .navigationSubtitle(pageSubtitle)
            .overlay(alignment: .bottom) {
                Group {
                    if let operation = model.operation {
                        StatusToast(text: operation, working: true)
                    } else if let notice = model.notice {
                        StatusToast(text: notice, working: false, dismiss: { model.notice = nil })
                    }
                }
                .padding(.bottom, 18)
                .transition(.move(edge: .bottom).combined(with: .opacity))
            }
            .animation(.snappy(duration: 0.28), value: model.operation)
            .animation(.snappy(duration: 0.28), value: model.notice)
            .task(id: model.notice) {
                // A result stays long enough to read, then gets out of the way.
                guard let shown = model.notice else { return }
                try? await Task.sleep(for: .seconds(7))
                if !Task.isCancelled, model.notice == shown { model.notice = nil }
            }
            .toolbar { windowToolbar }
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
            updateColumnVisibility()
            await warmNavigation()
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
            if let path = notification.object as? String { openProject(path, prepare: false) }
        }
        .onReceive(NotificationCenter.default.publisher(for: .devlineLibrary)) { _ in openLibrary(.assets) }
        .onReceive(NotificationCenter.default.publisher(for: .devlineShowPage)) { notification in
            if let name = notification.object as? String, let page = Workspace(rawValue: name) { workspace = page }
        }
        .onReceive(NotificationCenter.default.publisher(for: .devlineUpdates)) { _ in openLibrary(.updates) }
        .onReceive(NotificationCenter.default.publisher(for: .devlineActivity)) { _ in showingActivity = true }
        .onReceive(NotificationCenter.default.publisher(for: .devlineAppUpdates)) { _ in showingAppUpdates = true }
        .onChange(of: currentPage, initial: true) { _, page in
            if !mounted.contains(page) { mounted.insert(page) }
            if let path = projectContext?.path, page == Self.projectPage(path) { keepProjectPage(path) }
            // Nothing typed should land in a field on a page that is no longer on screen.
            if let window = NSApp.keyWindow, window.firstResponder is NSText { window.makeFirstResponder(nil) }
        }
        .onChange(of: workspace, initial: true) { _, page in
            UserDefaults.standard.set(page?.rawValue ?? Workspace.home.rawValue, forKey: "devline.workspace")
            updateColumnVisibility()
        }
        .onChange(of: librarySection) { _, section in
            UserDefaults.standard.set(section.rawValue, forKey: "devline.librarySection")
        }
        .onChange(of: focusReading) { _, _ in updateColumnVisibility() }
        .onChange(of: model.libraryRevision) { _, _ in
            if let path = projectContext?.path { Task { await loadProjectContext(path) } }
            Task { await infrastructure.refresh() }
        }
        .onOpenURL(perform: handleURL)
    }

    private func updateColumnVisibility() {
        let desired: NavigationSplitViewVisibility = focusReading && workspace == .docs ? .detailOnly : .all
        if columnVisibility != desired { columnVisibility = desired }
    }

    /// Each page's toolbar controls are made once and then only hidden or shown. Adding and
    /// removing toolbar items is the slowest part of changing page, so it is avoided where the
    /// system allows.
    @ToolbarContentBuilder private var windowToolbar: some ToolbarContent {
        let library = workspace == .library
        let services = workspace == .services && projectContext != nil
        if #available(macOS 15.0, *) {
            ToolbarItem(placement: .principal) { librarySectionPicker }.hidden(!library)
            ToolbarItem(placement: .principal) { servicesScopePicker }.hidden(!services)
            ToolbarItem(placement: .primaryAction) { newProjectButton }.hidden(!showingProject)
        } else {
            if library { ToolbarItem(placement: .principal) { librarySectionPicker } }
            if services { ToolbarItem(placement: .principal) { servicesScopePicker } }
            if showingProject { ToolbarItem(placement: .primaryAction) { newProjectButton } }
        }
    }

    private var librarySectionPicker: some View {
        Picker("Library Section", selection: $librarySection) {
            ForEach(LibrarySection.allCases, id: \.self) { Text($0.title).tag($0) }
        }.pickerStyle(.segmented).labelsHidden().fixedSize()
    }

    private var servicesScopePicker: some View {
        Picker("Services Section", selection: $showingProjectServices) {
            Text("This Project").tag(true)
            Text("All Services").tag(false)
        }.pickerStyle(.segmented).labelsHidden().fixedSize()
    }

    private var newProjectButton: some View {
        Button(action: beginNewProject) {
            Label("New Project", systemImage: "plus")
        }.help("New Project (⌘N)")
    }

    /// A project page needs a project; without one the window falls back to Home.
    private var showingProject: Bool { workspace == .project && projectContext != nil }

    private var pageTitle: String {
        switch workspace ?? .home {
        case .project: projectContext.map { URL(fileURLWithPath: $0.path).lastPathComponent } ?? Workspace.home.rawValue
        case let page: page.rawValue
        }
    }

    private var sidebarSelection: Binding<SidebarItem?> {
        Binding(
            get: {
                if let pendingProject { return .project(pendingProject) }
                if showingProject, let path = projectContext?.path {
                    return .project(model.recentProjects.first { sameProject($0.path, path) }?.path ?? path)
                }
                let page = workspace ?? .home
                return .page(page == .project ? .home : page)
            },
            set: { item in
                switch item {
                case .page(let page): workspace = page
                case .project(let path):
                    if sameProject(path, projectContext?.path) { workspace = .project } else { openProject(path, prepare: false) }
                case nil: break
                }
            }
        )
    }

    private static func projectPage(_ path: String) -> String { "project:" + path }

    /// The page on screen. Each one is built on its first visit and then kept.
    private var currentPage: String {
        switch workspace ?? .home {
        case .project:
            if let path = projectContext?.path { return Self.projectPage(path) }
            return "home"
        case .home: return "home"
        case .docs: return "docs"
        case .services: return projectContext != nil && showingProjectServices ? "services.project" : "services.all"
        case .library: return "library." + librarySection.rawValue
        }
    }

    private func shows(_ page: String) -> Bool { currentPage == page }
    private func keeps(_ page: String) -> Bool { currentPage == page || mounted.contains(page) }

    private var pages: some View {
        ZStack {
            if keeps("home") { kept("home") { homePage } }
            projectPageStack
            servicesPages
            if keeps("docs") { kept("docs") { docsContent } }
            libraryPages
        }
    }

    private func kept<Content: View>(_ page: String, @ViewBuilder _ content: @escaping () -> Content) -> some View {
        KeptPage(shown: shows(page), content: content).equatable()
    }

    private var homePage: some View {
        HomeView(model: model, inventory: runtimeInventory, infrastructure: infrastructure, shelves: shelves,
                 chooseFolder: chooseProjectFolder,
                 create: { newStarter = $0 }, showStarter: showStarter,
                 showDocs: { workspace = .docs }, showServices: { openServices(project: false) },
                 showPackages: { openLibrary(.packages) }, showToolchain: showToolchain,
                 startersRequest: startersRequest, online: connectivity.online, visible: shows("home"))
    }

    private var projectPageStack: some View {
        ForEach(shownProjectPages, id: \.self) { path in
            kept(Self.projectPage(path)) {
                if let context = projectContext?.path == path ? projectContext : knownProjects[path] {
                    ProjectWorkspaceView(model: model, infrastructure: infrastructure, project: context,
                        showDocs: browseProjectDocs, showDocset: browseProjectDocset,
                        showInfrastructure: { openServices(project: false) }, showStarter: showStarter,
                        showToolchains: { showToolchain(context.stack) }, showProjectServices: { openServices(project: true) },
                        closeProject: clearProject, online: connectivity.online, visible: shows(Self.projectPage(path)))
                }
            }
        }
    }

    @ViewBuilder private var servicesPages: some View {
        if let projectContext, keeps("services.project") {
            kept("services.project") {
                ProjectServicesView(model: model, infrastructure: infrastructure, project: projectContext,
                                    visible: shows("services.project"))
                    .id(projectContext.path)
            }
        }
        if keeps("services.all") {
            kept("services.all") {
                InfrastructureView(model: model, library: infrastructure, openDocs: openInfrastructureReference,
                                   openProjectServices: projectContext == nil ? nil : { openServices(project: true) },
                                   visible: shows("services.all"))
            }
        }
    }

    @ViewBuilder private var libraryPages: some View {
        ForEach(LibrarySection.allCases, id: \.self) { section in
            let page = "library." + section.rawValue
            if keeps(page) { kept(page) { libraryContent(section) } }
        }
    }

    private var shownProjectPages: [String] {
        guard workspace == .project, let path = projectContext?.path, !projectPages.contains(path) else { return projectPages }
        return projectPages + [path]
    }

    /// Keeps the pages of the projects opened most recently, so returning to one is immediate.
    private func keepProjectPage(_ path: String) {
        guard projectPages.last != path else { return }
        projectPages.removeAll { $0 == path }
        projectPages.append(path)
        if projectPages.count > 6 { projectPages.removeFirst(projectPages.count - 6) }
    }

    private var pageSubtitle: String {
        switch workspace ?? .home {
        case .project where showingProject: projectContext.map { $0.stacks.map(displayStack).joined(separator: " · ") } ?? ""
        case .library: librarySection.title
        default: ""
        }
    }

    /// What Devline is doing in the background, and whether this Mac is offline.
    @ViewBuilder private var sidebarStatus: some View {
        let active = model.jobs.filter(\.isActive)
        if !active.isEmpty || !connectivity.online {
            VStack(alignment: .leading, spacing: 8) {
                if let job = active.first {
                    Button { showingActivity = true } label: {
                        HStack(spacing: 9) {
                            ProgressView().controlSize(.small)
                            VStack(alignment: .leading, spacing: 1) {
                                Text(active.count > 1 ? "\(job.title) + \(active.count - 1) more" : job.title)
                                    .font(.caption.weight(.medium)).lineLimit(1)
                                Text(job.stage).font(.caption2).foregroundStyle(.secondary).lineLimit(1)
                            }
                            Spacer(minLength: 0)
                        }.contentShape(Rectangle())
                    }
                    .buttonStyle(.plain).help("Show progress and logs")
                }
                if !connectivity.online {
                    Label("Offline. Working from this Mac.", systemImage: "wifi.slash")
                        .font(.caption).foregroundStyle(.secondary)
                        .help("Docs, starters, saved packages, and saved service images keep working. Downloads wait for a connection.")
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.horizontal, 14).padding(.vertical, 10)
            .transition(.opacity)
        }
    }

    private var docsContent: some View {
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
            isActive: workspace == .docs,
            openLibrary: { openDocsManager() },
            installDocset: { openDocsManager(install: true, packID: $0) }
        )
    }

    @ViewBuilder private func libraryContent(_ section: LibrarySection) -> some View {
        switch section {
        case .assets:
            if let catalog = model.catalog {
                LibraryView(model: model, infrastructure: infrastructure, catalog: catalog,
                            projectPath: projectContext?.path, isActive: showingAssets,
                            manageDocs: { openDocsManager() })
            }
        case .starters:
            TemplatesView(model: model, selection: $starterReference, newStarter: $newStarter, openDocs: browseDocset)
        case .packages:
            PackagesView(model: model, shelves: shelves, inventory: runtimeInventory, showToolchain: showToolchain,
                         visible: shows("library.Packages"))
        case .toolchains:
            RuntimesView(
                model: model,
                inventory: runtimeInventory,
                infrastructure: infrastructure,
                selection: $toolchainSelection,
                projectStack: projectContext?.stack,
                visible: shows("library.Toolchains"),
                showStarter: showStarter,
                showDocs: { stack in
                    projectScopeRequested = false
                    directDocumentID = nil
                    docStack = stack
                    docPack = nil
                    query = ""
                    lookupRevision += 1
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

    private func showToolchain(_ id: String?) {
        if let id { toolchainSelection = id }
        openLibrary(.toolchains)
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
        if workspace == .library && librarySection == .starters,
           let starter = model.catalog?.templates.first(where: { $0.reference == starterReference && $0.verified }) {
            newStarter = starter
        } else {
            workspace = .home
            startersRequest += 1
        }
    }

    /// Loads what the other pages show before they are visited, so every first click is instant.
    private func warmNavigation() async {
        async let toolchains: Void = runtimeInventory.load(force: false)
        async let services: Void = infrastructure.loadIfNeeded(after: model.completedInfrastructureJobIDs)
        async let packages: Void = shelves.loadIfNeeded(after: model.completedPackageJobIDs)
        _ = await (toolchains, services, packages)
        for recent in model.recentProjects where !Task.isCancelled {
            guard FileManager.default.fileExists(atPath: recent.path) else { continue }
            if knownProjects[recent.path] == nil,
               let context: ProjectContext = try? await Core.request(CoreRequest("project.context", path: recent.path), priority: .utility) {
                knownProjects[context.path] = context
                knownProjects[recent.path] = context
            }
            await prefetchProjectReadiness(knownProjects[recent.path]?.path ?? recent.path)
        }
        await buildPagesAhead()
    }

    /// Builds the other pages one at a time while the window is idle, so the first visit to
    /// each is as quick as every later one.
    private func buildPagesAhead() async {
        var ahead = ["docs", "library.\(librarySection.rawValue)", "services.all"]
        ahead += LibrarySection.allCases.map { "library." + $0.rawValue }
        if projectContext != nil { ahead.append("services.project") }
        for page in ahead where !mounted.contains(page) {
            try? await Task.sleep(for: .milliseconds(300))
            guard !Task.isCancelled else { return }
            mounted.insert(page)
        }
        for recent in model.recentProjects.prefix(5) {
            guard let path = knownProjects[recent.path]?.path, !projectPages.contains(path) else { continue }
            try? await Task.sleep(for: .milliseconds(300))
            guard !Task.isCancelled else { return }
            if !projectPages.contains(path) { projectPages.insert(path, at: 0) }
        }
    }

    private func forgetProject(_ path: String) {
        if sameProject(path, projectContext?.path) { clearProject() }
        projectPages.removeAll { sameProject($0, path) }
        model.forgetProject(path)
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
        if reading.request != nil { reading.request = nil }
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
            knownProjects[context.path] = context
            UserDefaults.standard.set(context.path, forKey: "devline.projectPath")
            return true
        } catch is CancellationError {
            return false
        } catch {
            if projectRequest == request, navigation == nil || navigationRequest == navigation { model.error = error.localizedDescription }
            return false
        }
    }

    /// Shows a project immediately, then reads it again in the background. `prepare` also gets
    /// what it is missing onto this Mac, which is for a project that was just opened or created,
    /// not for switching between projects in the sidebar.
    private func openProject(_ path: String, prepare: Bool = true) {
        let navigation = beginNavigation()
        openingProject = true
        pendingProject = path
        if FileManager.default.fileExists(atPath: path) {
            // Navigation never waits: use what is known, or the bare folder, until the read lands.
            projectContext = knownProjects.first { sameProject($0.key, path) }?.value
                ?? ProjectContext(path: path, stacks: [], stack: nil, dependencies: nil, pins: nil, docsets: nil, origin: nil)
            workspace = .project
        }
        Task {
            defer {
                if navigationRequest == navigation { openingProject = false }
                if pendingProject == path { pendingProject = nil }
            }
            guard await loadProjectContext(path, navigation: navigation), navigationRequest == navigation else {
                // The folder could not be read as a project; do not leave a hollow page behind.
                if navigationRequest == navigation, projectContext?.stacks.isEmpty == true, projectContext?.dependencies == nil {
                    projectContext = nil
                    workspace = .home
                }
                return
            }
            docStack = projectContext?.stack.flatMap { stack in
                model.catalog?.docs.contains { $0.installed && $0.stack == stack } == true ? stack : nil
            }
            directDocumentID = nil
            docPack = nil
            query = ""
            projectScopeRequested = false
            lookupRevision += 1
            workspace = .project
            pendingProject = nil
            guard let resolvedPath = projectContext?.path else { return }
            model.rememberProject(resolvedPath)
            guard prepare else { return }
            do {
                let readiness: ToolboxStatus = try await Core.request(CoreRequest("toolbox.status", path: resolvedPath))
                guard navigationRequest == navigation, projectContext?.path == resolvedPath else { return }
                if !readiness.readyOffline {
                    guard !model.jobs.contains(where: { $0.isActive && $0.targetPath == resolvedPath && ["toolbox_prepare", "project_prepare"].contains($0.kind) }) else { return }
                    // Without a connection there is nothing to fetch; the project page says what is missing.
                    guard connectivity.online else { return }
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
        if workspace == .project { workspace = .home }
    }

    /// Opens one of the project's matched docsets, keeping project links and versions in scope.
    private func browseProjectDocset(_ reference: String) {
        guard let pack = model.catalog?.docs.first(where: { $0.reference == reference && $0.installed }) else {
            browseProjectDocs()
            return
        }
        beginNavigation()
        projectScopeRequested = true
        query = ""
        directDocumentID = nil
        docStack = pack.stack
        docPack = pack.reference
        lookupRevision += 1
        focusReading = false
        workspace = .docs
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
        openLibrary(.starters)
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
