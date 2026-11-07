import AppKit
import SwiftUI

private extension Notification.Name {
    static let devkitFind = Self("DevKit.find")
    static let devkitNew = Self("DevKit.new")
    static let devkitOpenProject = Self("DevKit.openProject")
    static let devlineOpenRecent = Self("Devline.openRecent")
    static let devkitLibrary = Self("DevKit.library")
    static let devkitUpdates = Self("DevKit.updates")
}

enum Workspace: String, CaseIterable, Identifiable {
    case toolbox = "Toolbox"
    case project = "Current Project"
    case infrastructure = "Infrastructure"
    case docs = "Docs"
    case templates = "Templates"
    case runtimes = "Runtimes"
    case library = "Library"
    case updates = "Updates"
    case activity = "Activity"
    case offline = "Ready Offline"

    var id: String { rawValue }
    var title: String {
        switch self {
        case .docs: "Documentation"
        case .templates: "App Starters"
        case .runtimes: "Toolchains"
        case .library: "Docset Library"
        case .offline: "Offline Library"
        default: rawValue
        }
    }
    var icon: String {
        switch self {
        case .toolbox: "shippingbox"
        case .project: "folder"
        case .infrastructure: "server.rack"
        case .docs: "book.closed"
        case .templates: "square.on.square"
        case .runtimes: "terminal"
        case .library: "tray.full"
        case .updates: "arrow.down.circle"
        case .activity: "clock.arrow.circlepath"
        case .offline: "checkmark.shield"
        }
    }
}

@main
struct DevKitMacApp: App {
    @NSApplicationDelegateAdaptor(DevlineAppDelegate.self) private var appDelegate
    @StateObject private var appUpdates = AppUpdates.shared
    @StateObject private var model = WorkspaceModel()
    @AppStorage("devline.appearance") private var appearance = "system"

    var body: some Scene {
        Window("Devline", id: "workspace") {
            WorkspaceView(model: model)
                .preferredColorScheme(appearance == "system" ? nil : appearance == "light" ? .light : .dark)
        }
        .defaultSize(width: 1120, height: 720)
        .windowResizability(.contentMinSize)
        .commands {
            CommandGroup(after: .appInfo) {
                if appUpdates.configured {
                    Button("Check for App Updates…", action: appUpdates.check).disabled(!appUpdates.canCheck)
                } else {
                    Button("App Updates and Downloads…") { announce(.devkitUpdates) }
                }
                Button("Documentation, Starter, and Image Updates…") { announce(.devkitUpdates) }
            }
            CommandGroup(replacing: .newItem) {
                Button("New Project…") { announce(.devkitNew) }
                    .keyboardShortcut("n")
                Button("Open Project Folder…") { announce(.devkitOpenProject) }
                    .keyboardShortcut("o")
                Menu("Open Recent") {
                    if model.recentProjects.isEmpty { Text("No Recent Projects") }
                    ForEach(model.recentProjects) { project in
                        Button(project.name) { NotificationCenter.default.post(name: .devlineOpenRecent, object: project.path) }
                            .help(project.path)
                    }
                    if !model.recentProjects.isEmpty {
                        Divider()
                        Button("Clear Menu") { model.clearRecentProjects() }
                    }
                }
            }
            CommandMenu("Workspace") {
                Button("Find in Docs") { announce(.devkitFind) }
                    .keyboardShortcut("f")
                Button("Open Library") { announce(.devkitLibrary) }
                    .keyboardShortcut("l", modifiers: [.command, .shift])
            }
        }

        Settings {
            SettingsView(model: model)
                .preferredColorScheme(appearance == "system" ? nil : appearance == "light" ? .light : .dark)
        }
    }

    private func announce(_ action: Notification.Name) {
        NotificationCenter.default.post(name: action, object: nil)
    }
}

@MainActor
final class WorkspaceModel: ObservableObject {
    @Published private(set) var catalog: Catalog?
    @Published private(set) var catalogError: String?
    @Published private(set) var loadingCatalog = false
    @Published private(set) var libraryRevision = 0
    @Published private(set) var operation: String?
    @Published var error: String?
    @Published var notice: String?
    @Published private(set) var jobs: [JobRecord] = []
    @Published var inspectedJob: JobRecord?
    private var jobMonitor: Task<Void, Never>?
    private var pendingServicePreparations: [String: String] =
        UserDefaults.standard.dictionary(forKey: "devline.pendingServicePreparations") as? [String: String] ?? [:]
    private var continuingServicePreparation = false
    @Published private(set) var recentProjects: [RecentProject] = {
        guard let data = UserDefaults.standard.data(forKey: "devline.recentProjects") else { return [] }
        return (try? JSONDecoder().decode([RecentProject].self, from: data)) ?? []
    }()

    func rememberProject(_ path: String) {
        recentProjects = RecentProject.remembering(path, in: recentProjects)
        saveRecentProjects()
    }
    func forgetProject(_ path: String) {
        recentProjects.removeAll { $0.path == path }
        saveRecentProjects()
    }
    func clearRecentProjects() { recentProjects = []; saveRecentProjects() }
    private func saveRecentProjects() {
        if let data = try? JSONEncoder().encode(recentProjects) {
            UserDefaults.standard.set(data, forKey: "devline.recentProjects")
        }
    }

    func refresh() async {
        do { try await reload(); await refreshJobs() }
        catch is CancellationError {}
        catch { if catalog != nil { self.error = error.localizedDescription } }
    }

    @discardableResult
    func startJob(_ request: CoreRequest, inspect: Bool = true) async -> Bool {
        do {
            let job: JobRecord = try await Core.request(request)
            if job.kind == "infra_stack_add", let path = job.targetPath {
                pendingServicePreparations[job.id] = path
                UserDefaults.standard.set(pendingServicePreparations, forKey: "devline.pendingServicePreparations")
            }
            if inspect { inspectedJob = job }
            await refreshJobs()
            return true
        } catch { self.error = error.localizedDescription; return false }
    }

    func refreshJobs() async {
        do { jobs = try await Core.request(CoreRequest("jobs.list")); await continueServicePreparation() }
        catch { return }
        guard jobMonitor == nil, jobs.contains(where: \.isActive) else { return }
        jobMonitor = Task { @MainActor in
            defer { jobMonitor = nil }
            while !Task.isCancelled {
                do { try await Task.sleep(for: .seconds(1)) } catch { return }
                let active = Set(jobs.filter(\.isActive).map(\.id))
                do {
                    jobs = try await Core.request(CoreRequest("jobs.list"))
                    await continueServicePreparation()
                    if jobs.contains(where: { active.contains($0.id) && !$0.isActive }) { try await reload() }
                } catch { self.error = error.localizedDescription; return }
                if !jobs.contains(where: \.isActive) { return }
            }
        }
    }

    private func continueServicePreparation() async {
        guard !continuingServicePreparation else { return }
        continuingServicePreparation = true
        defer { continuingServicePreparation = false }
        var startedPreparation = false
        for job in jobs where !job.isActive {
            guard let path = pendingServicePreparations[job.id] else { continue }
            if job.state == "succeeded", !job.needsAttention {
                do {
                    let _: JobRecord = try await Core.request(CoreRequest("jobs.start", path: path, kind: "toolbox_prepare"))
                    startedPreparation = true
                } catch { self.error = error.localizedDescription; continue }
            }
            pendingServicePreparations.removeValue(forKey: job.id)
            UserDefaults.standard.set(pendingServicePreparations, forKey: "devline.pendingServicePreparations")
        }
        if startedPreparation, let updated: [JobRecord] = try? await Core.request(CoreRequest("jobs.list")) {
            jobs = updated
        }
    }

    @discardableResult
    func mutate(_ requests: [CoreRequest], label: String, completion: String) async -> Bool {
        guard operation == nil else { return false }
        operation = label
        notice = nil
        defer { operation = nil }

        do {
            for (index, request) in requests.enumerated() {
                if requests.count > 1 { operation = "\(label) (\(index + 1) of \(requests.count))" }
                let _: Acknowledgment = try await Core.request(request)
            }
            try await reload()
            notice = completion
            return true
        } catch {
            try? await reload()
            self.error = error.localizedDescription
            return false
        }
    }

    private func reload() async throws {
        loadingCatalog = true
        defer { loadingCatalog = false }
        do {
            let updated: Catalog = try await Core.request(CoreRequest("catalog"))
            let previous = catalog?.docs.filter(\.installed).map { "\($0.reference):\($0.active == true)" }
            let next = updated.docs.filter(\.installed).map { "\($0.reference):\($0.active == true)" }
            catalog = updated
            catalogError = nil
            if previous != next { libraryRevision += 1 }
        } catch is CancellationError { throw CancellationError() }
        catch { catalogError = error.localizedDescription; throw error }
    }
}

private struct WorkspaceView: View {
    @Environment(\.scenePhase) private var scenePhase
    @Environment(\.openWindow) private var openWindow
    @ObservedObject var model: WorkspaceModel
    @StateObject private var reading = ReadingSession()
    @StateObject private var infrastructure = InfrastructureLibrary()
    @State private var workspace: Workspace? = Workspace(rawValue: UserDefaults.standard.string(forKey: "devline.workspace") ?? "Toolbox") ?? .toolbox
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
    @State private var starterReference: String?
    @State private var newStarter: Starter?
    @State private var updateCheckRequest = 0

    var body: some View {
        NavigationSplitView(columnVisibility: $columnVisibility) {
            List(selection: $workspace) {
                Section("Workspace") {
                    sidebarRows([.toolbox, .project])
                }
                Section("Build") { sidebarRows([.templates, .infrastructure]) }
                Section("Reference") { sidebarRows([.docs, .runtimes]) }
                Section("Manage") { sidebarRows([.library, .offline, .updates, .activity]) }
            }
            .listStyle(.sidebar)
            .safeAreaInset(edge: .bottom) {
                if let catalog = model.catalog {
                    VStack(alignment: .leading, spacing: 5) {
                        Divider()
                        Label("\(Set(catalog.docs.filter(\.installed).map(\.id)).count) docsets · \(catalog.docs.filter(\.installed).count) versions", systemImage: "internaldrive")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                            .padding(.horizontal, 15)
                            .padding(.bottom, 10)
                    }
                }
            }
            .navigationSplitViewColumnWidth(min: 165, ideal: 190, max: 245)
        } detail: {
            Group {
                if model.catalog == nil {
                    CatalogLoadingView(model: model)
                } else {
                    workspaceContent
                }
            }
            .navigationTitle((workspace ?? .toolbox).title)
            .safeAreaInset(edge: .bottom, spacing: 0) {
                if let operation = model.operation {
                    StatusBar(text: operation, working: true)
                } else if let notice = model.notice {
                    StatusBar(text: notice, working: false, dismiss: { model.notice = nil })
                }
            }
            .toolbar {
                ToolbarItem(placement: .secondaryAction) {
                    Button(action: chooseProjectFolder) {
                        Label("Open Project Folder", systemImage: "folder")
                    }
                    .help("Open Project Folder (⌘O)")
                    .disabled(openingProject)
                }
                ToolbarItem(placement: .primaryAction) {
                    Button(action: beginNewProject) {
                        Label("New Project", systemImage: "plus")
                    }
                    .help("New Project (⌘N)")
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
        .alert("Devline", isPresented: Binding(
            get: { model.error != nil },
            set: { if !$0 { model.error = nil } }
        )) {
            Button("OK", role: .cancel) { model.error = nil }
        } message: {
            Text(model.error ?? "")
        }
        .onReceive(NotificationCenter.default.publisher(for: .devkitFind)) { _ in
            beginNavigation()
            workspace = .docs
            directDocumentID = nil
            focusRequest += 1
        }
        .onReceive(NotificationCenter.default.publisher(for: .devkitNew)) { _ in beginNewProject() }
        .onReceive(NotificationCenter.default.publisher(for: .devkitOpenProject)) { _ in chooseProjectFolder() }
        .onReceive(NotificationCenter.default.publisher(for: .devlineOpenRecent)) { notification in
            if let path = notification.object as? String { openProject(path) }
        }
        .onReceive(NotificationCenter.default.publisher(for: .devkitLibrary)) { _ in workspace = .library }
        .onReceive(NotificationCenter.default.publisher(for: .devkitUpdates)) { _ in
            updateCheckRequest += 1
            workspace = .updates
        }
        .onChange(of: workspace) { _, page in
            UserDefaults.standard.set(page?.rawValue ?? "Toolbox", forKey: "devline.workspace")
            columnVisibility = focusReading && page == .docs ? .detailOnly : .all
            if page != .updates { updateCheckRequest = 0 }
        }
        .onChange(of: focusReading) { _, focused in columnVisibility = focused && workspace == .docs ? .detailOnly : .all }
        .onChange(of: model.libraryRevision) { _, _ in
            if let path = projectContext?.path { Task { await loadProjectContext(path) } }
        }
        .onOpenURL(perform: handleURL)
    }

    @ViewBuilder
    private var workspaceContent: some View {
        switch workspace ?? .toolbox {
        case .toolbox:
            ToolboxView(model: model, infrastructure: infrastructure, project: projectContext,
                navigate: { workspace = $0 }, openProject: chooseProjectFolder, openRecent: openProject)
        case .project:
            ProjectWorkspaceView(model: model, infrastructure: infrastructure, project: projectContext,
                openProject: chooseProjectFolder, newProject: beginNewProject, showDocs: browseProjectDocs,
                showInfrastructure: { workspace = .infrastructure }, showStarter: showStarter,
                showToolchains: { workspace = .runtimes }, closeProject: clearProject)
                .id(projectContext?.path)
        case .infrastructure:
            InfrastructureView(model: model, library: infrastructure, openDocs: openInfrastructureReference)
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
                openLibrary: { workspace = .library }
            )
        case .templates:
            TemplatesView(model: model, selection: $starterReference, newStarter: $newStarter, openDocs: browseDocset)
        case .runtimes:
            RuntimesView(
                model: model,
                projectStack: projectContext?.stack,
                showStarter: showStarter,
                showDocs: { stack in
                    directDocumentID = nil
                    docStack = stack
                    docPack = nil
                    query = ""
                    workspace = .docs
                    focusRequest += 1
                },
                showLibrary: { workspace = .library }
            )
        case .library:
            LibraryView(model: model)
        case .updates:
            UpdatesView(model: model, checkOnOpen: updateCheckRequest > 0)
                .id(updateCheckRequest)
        case .activity:
            ActivityView(model: model)
        case .offline:
            OfflineLibraryView(model: model, infrastructure: infrastructure, projectPath: projectContext?.path)
        }
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
        reading.open(location)
        directDocumentID = id
        workspace = .docs
    }

    private func sidebarRows(_ pages: [Workspace]) -> some View {
        ForEach(pages) { page in Label(page.title, systemImage: page.icon).tag(page) }
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
            reading.open(ReferenceLocation(id: reference))
            directDocumentID = reference
            workspace = .docs
        }
    }

    private func beginNewProject() {
        if workspace == .templates,
           let starter = model.catalog?.templates.first(where: { $0.reference == starterReference && $0.isInstalled }) {
            newStarter = starter
        } else {
            workspace = .templates
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
            workspace = .project
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
        UserDefaults.standard.removeObject(forKey: "devline.projectPath")
        docStack = nil
        docPack = nil
    }

    private func browseProjectDocs() {
        beginNavigation()
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
        workspace = .templates
    }

    private func lookUp(_ request: LookupRequest) {
        let navigation = beginNavigation()
        if let path = request.path {
            openingProject = true
            Task {
                defer { if navigationRequest == navigation { openingProject = false } }
                guard await loadProjectContext(path, navigation: navigation), navigationRequest == navigation else { return }
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

extension Starter: Identifiable {}

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

struct StatusBar: View {
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

private struct SettingsView: View {
    @ObservedObject var model: WorkspaceModel
    @AppStorage("devline.appearance") private var appearance = "system"
    @State private var cliLocation: String?
    @State private var cliError: String?
    @State private var pathCopied = false

    var body: some View {
        Form {
            Section("Appearance") {
                Picker("Appearance", selection: $appearance) {
                    Text("System").tag("system")
                    Text("Light").tag("light")
                    Text("Dark").tag("dark")
                }.pickerStyle(.segmented)
                Text("Text size is available in the reader’s Reading Options menu.")
                    .font(.caption).foregroundStyle(.secondary)
            }
            Section("Local Library") {
                if let home = model.catalog?.home {
                    LabeledContent("Folder", value: home)
                        .textSelection(.enabled)
                    Button("Show in Finder") {
                        NSWorkspace.shared.open(URL(fileURLWithPath: home, isDirectory: true))
                    }
                }
                Button("Refresh Library") { Task { await model.refresh() } }
            }
            AppUpdatesSettings()
            LookupSettings()
            Section("Command Line") {
                Text("Link the bundled devline command into ~/.local/bin to use it from Terminal.")
                    .font(.callout).foregroundStyle(.secondary)
                HStack {
                    Button("Install CLI", action: installCLI)
                    Button(pathCopied ? "PATH Command Copied" : "Copy PATH Command", action: copyPathCommand)
                        .help("Copy a command that adds ~/.local/bin to this Terminal session")
                }
                if let cliLocation {
                    Label("Available at \(cliLocation)", systemImage: "checkmark.circle")
                        .font(.callout).textSelection(.enabled)
                }
                if let cliError {
                    Label(cliError, systemImage: "exclamationmark.circle")
                        .font(.callout).foregroundStyle(.red)
                }
            }
            Section("Keyboard Shortcuts") {
                LabeledContent("Find in Docs", value: "⌘F")
                LabeledContent("Find in Page", value: "⌥⌘F")
                LabeledContent("Page Contents", value: "⇧⌘O")
                LabeledContent("Focus Reading", value: "⇧⌘R")
                LabeledContent("Save Reference", value: "⌘D")
                LabeledContent("Saved and Recent", value: "⇧⌘B")
                LabeledContent("Back / Forward", value: "⌘[ / ⌘]")
                LabeledContent("New Project", value: "⌘N")
                LabeledContent("Open Project Folder", value: "⌘O")
                LabeledContent("Open Library", value: "⇧⌘L")
            }
        }
        .formStyle(.grouped)
        .frame(width: 550, height: 560)
    }

    private func installCLI() {
        cliLocation = nil
        cliError = nil
        guard let executable = Bundle.main.executableURL else {
            cliError = "Open the packaged Devline.app to install the CLI."
            return
        }
        let target = executable.deletingLastPathComponent().appendingPathComponent("devline")
        let directory = FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent(".local/bin", isDirectory: true)
        do {
            cliLocation = try installDevKitLink(target: target, directory: directory).path(percentEncoded: false)
        } catch {
            cliError = error.localizedDescription
        }
    }

    private func copyPathCommand() {
        NSPasteboard.general.clearContents()
        pathCopied = NSPasteboard.general.setString(#"export PATH="$HOME/.local/bin:$PATH""#, forType: .string)
    }
}

func installDevKitLink(target: URL, directory: URL) throws -> URL {
    let files = FileManager.default
    let targetPath = target.path(percentEncoded: false)
    guard (try? target.resourceValues(forKeys: [.isRegularFileKey]).isRegularFile) == true,
          files.isExecutableFile(atPath: targetPath) else {
        throw CoreError(message: "Open the packaged Devline.app to install its bundled CLI.")
    }
    try files.createDirectory(at: directory, withIntermediateDirectories: true)
    let destination = directory.appendingPathComponent("devline")
    let path = destination.path(percentEncoded: false)
    if let previous = try? files.destinationOfSymbolicLink(atPath: path) {
        let linked = URL(fileURLWithPath: previous, relativeTo: directory).absoluteURL
        if linked.resolvingSymlinksInPath().path == target.resolvingSymlinksInPath().path {
            return destination
        }
        throw CoreError(message: "A different link already exists at \(path). Choose how to manage it before installing Devline.")
    }
    guard !files.fileExists(atPath: path) else {
        throw CoreError(message: "A file already exists at \(path). Choose how to manage it before installing Devline.")
    }
    try files.createSymbolicLink(at: destination, withDestinationURL: target)
    return destination
}
