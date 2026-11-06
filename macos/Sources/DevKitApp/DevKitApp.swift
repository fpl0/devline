import AppKit
import SwiftUI

private extension Notification.Name {
    static let devkitFind = Self("DevKit.find")
    static let devkitNew = Self("DevKit.new")
    static let devkitOpenProject = Self("DevKit.openProject")
    static let devkitLibrary = Self("DevKit.library")
    static let devkitUpdates = Self("DevKit.updates")
}

enum Workspace: String, CaseIterable, Identifiable {
    case docs = "Docs"
    case templates = "Templates"
    case runtimes = "Runtimes"
    case library = "Library"
    case updates = "Updates"

    var id: String { rawValue }
    var icon: String {
        switch self {
        case .docs: "book.closed"
        case .templates: "square.on.square"
        case .runtimes: "terminal"
        case .library: "tray.full"
        case .updates: "arrow.down.circle"
        }
    }
}

@main
struct DevKitMacApp: App {
    @StateObject private var model = WorkspaceModel()

    var body: some Scene {
        Window("Devline", id: "workspace") {
            WorkspaceView(model: model)
        }
        .defaultSize(width: 1120, height: 720)
        .windowResizability(.contentMinSize)
        .commands {
            CommandGroup(after: .appInfo) {
                Button("Check for Updates…") { announce(.devkitUpdates) }
            }
            CommandGroup(replacing: .newItem) {
                Button("New Project…") { announce(.devkitNew) }
                    .keyboardShortcut("n")
                Button("Open Project Folder…") { announce(.devkitOpenProject) }
                    .keyboardShortcut("o")
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
        }
    }

    private func announce(_ action: Notification.Name) {
        NotificationCenter.default.post(name: action, object: nil)
    }
}

@MainActor
final class WorkspaceModel: ObservableObject {
    @Published private(set) var catalog: Catalog?
    @Published private(set) var libraryRevision = 0
    @Published private(set) var operation: String?
    @Published var error: String?
    @Published var notice: String?

    func refresh() async {
        do { try await reload() }
        catch is CancellationError {}
        catch { self.error = error.localizedDescription }
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
        let updated: Catalog = try await Core.request(CoreRequest("catalog"))
        let previous = catalog?.docs.filter(\.installed).map(\.reference)
        let next = updated.docs.filter(\.installed).map(\.reference)
        catalog = updated
        if previous != next { libraryRevision += 1 }
    }
}

private struct WorkspaceView: View {
    @Environment(\.scenePhase) private var scenePhase
    @ObservedObject var model: WorkspaceModel
    @State private var workspace: Workspace? = .docs
    @State private var query = ""
    @State private var docStack: String?
    @State private var directDocumentID: String?
    @State private var projectContext: ProjectContext?
    @State private var openingProject = false
    @State private var focusRequest = 0
    @State private var starterReference: String?
    @State private var newStarter: Starter?
    @State private var updateCheckRequest = 0

    var body: some View {
        NavigationSplitView {
            List(selection: $workspace) {
                Section("Workspace") {
                    ForEach(Workspace.allCases) { page in
                        Label(page.rawValue, systemImage: page.icon)
                            .tag(page)
                    }
                }
            }
            .listStyle(.sidebar)
            .safeAreaInset(edge: .bottom) {
                if let catalog = model.catalog {
                    VStack(alignment: .leading, spacing: 5) {
                        Divider()
                        Label("\(catalog.docs.filter(\.installed).count) doc packs installed", systemImage: "internaldrive")
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
                switch workspace ?? .docs {
                case .docs:
                    DocsView(
                        model: model,
                        query: $query,
                        stack: $docStack,
                        directDocumentID: $directDocumentID,
                        projectContext: projectContext,
                        clearProject: clearProject,
                        focusRequest: focusRequest,
                        openLibrary: { workspace = .library }
                    )
                case .templates:
                    TemplatesView(model: model, selection: $starterReference, newStarter: $newStarter)
                case .runtimes:
                    RuntimesView(
                        model: model,
                        projectStack: projectContext?.stack,
                        showStarter: { reference in
                            starterReference = reference
                            workspace = .templates
                        },
                        showDocs: { stack in
                            directDocumentID = nil
                            docStack = stack
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
                }
            }
            .navigationTitle((workspace ?? .docs).rawValue)
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
        .task(id: scenePhase) {
            if scenePhase == .active { await model.refresh() }
        }
        .sheet(item: $newStarter) { starter in
            NewProjectView(starter: starter) { path in
                newStarter = nil
                Task { await loadProjectContext(path) }
            }
        }
        .alert("Devline", isPresented: Binding(
            get: { model.error != nil },
            set: { if !$0 { model.error = nil } }
        )) {
            Button("OK", role: .cancel) { model.error = nil }
        } message: {
            Text(model.error ?? "")
        }
        .onReceive(NotificationCenter.default.publisher(for: .devkitFind)) { _ in
            workspace = .docs
            directDocumentID = nil
            focusRequest += 1
        }
        .onReceive(NotificationCenter.default.publisher(for: .devkitNew)) { _ in beginNewProject() }
        .onReceive(NotificationCenter.default.publisher(for: .devkitOpenProject)) { _ in chooseProjectFolder() }
        .onReceive(NotificationCenter.default.publisher(for: .devkitLibrary)) { _ in workspace = .library }
        .onReceive(NotificationCenter.default.publisher(for: .devkitUpdates)) { _ in
            updateCheckRequest += 1
            workspace = .updates
        }
        .onChange(of: workspace) { _, page in
            if page != .updates { updateCheckRequest = 0 }
        }
        .onOpenURL { url in
            guard ["devline", "devkit"].contains(url.scheme ?? ""), url.host == "docs",
                  let components = URLComponents(url: url, resolvingAgainstBaseURL: false),
                  let id = components.queryItems?.first(where: { $0.name == "id" })?.value,
                  !id.isEmpty else { return }
            directDocumentID = id
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
            Task { await loadProjectContext(url.path(percentEncoded: false)) }
        }
    }

    private func loadProjectContext(_ path: String) async {
        guard !openingProject else { return }
        openingProject = true
        defer { openingProject = false }
        do {
            let context: ProjectContext = try await Core.request(CoreRequest("project.context", path: path))
            projectContext = context
            if let stack = context.stack, model.catalog?.docs.contains(where: { $0.installed && $0.stack == stack }) == true {
                docStack = stack
            } else {
                docStack = nil
            }
            query = ""
            directDocumentID = nil
            workspace = .docs
            focusRequest += 1
        } catch {
            model.error = error.localizedDescription
        }
    }

    private func clearProject() {
        projectContext = nil
        docStack = nil
    }
}

extension Starter: Identifiable {}

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
    @State private var cliLocation: String?
    @State private var cliError: String?
    @State private var pathCopied = false

    var body: some View {
        Form {
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
                LabeledContent("New Project", value: "⌘N")
                LabeledContent("Open Project Folder", value: "⌘O")
                LabeledContent("Open Library", value: "⇧⌘L")
            }
        }
        .formStyle(.grouped)
        .frame(width: 550, height: 420)
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
