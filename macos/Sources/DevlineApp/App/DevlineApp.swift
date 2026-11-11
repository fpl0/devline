import AppKit
import SwiftUI

extension Notification.Name {
    static let devlineFind = Self("Devline.find")
    static let devlineNew = Self("Devline.new")
    static let devlineOpenProject = Self("Devline.openProject")
    static let devlineOpenRecent = Self("Devline.openRecent")
    static let devlineLibrary = Self("Devline.library")
    static let devlineUpdates = Self("Devline.updates")
    static let devlineAppUpdates = Self("Devline.appUpdates")
    static let devlineActivity = Self("Devline.activity")
    static let devlineShowPage = Self("Devline.showPage")
}

@main
struct DevlineMacApp: App {
    @NSApplicationDelegateAdaptor(DevlineAppDelegate.self) private var appDelegate
    @StateObject private var appUpdates = AppUpdates.shared
    // Workspace views observe their model. Application commands only need the
    // recent projects, so catalog and job updates don't rebuild the scenes.
    @State private var model = WorkspaceModel()

    var body: some Scene {
        Window("Devline", id: "workspace") {
            WorkspaceView(model: model).preferredAppearance()
        }
        .defaultSize(width: 1120, height: 720)
        .windowResizability(.contentMinSize)
        .commands {
            CommandGroup(after: .appInfo) {
                if appUpdates.configured {
                    Button("Check for App Updates…", action: appUpdates.check).disabled(!appUpdates.canCheck)
                } else {
                    Button("App Updates and Downloads…") { announce(.devlineAppUpdates) }
                }
                Button("Library Updates…") { announce(.devlineUpdates) }
            }
            CommandGroup(replacing: .newItem) {
                Button("New Project…") { announce(.devlineNew) }
                    .keyboardShortcut("n")
                Button("Open Project Folder…") { announce(.devlineOpenProject) }
                    .keyboardShortcut("o")
                Menu("Open Recent") { OpenRecentItems(model: model) }
            }
            CommandMenu("Go") {
                Button("Home") { show("Home") }.keyboardShortcut("1")
                Button("Docs") { show("Docs") }.keyboardShortcut("2")
                Button("Services") { show("Services") }.keyboardShortcut("3")
                Button("Library") { show("Library") }.keyboardShortcut("4")
                Divider()
                Button("Find in Docs") { announce(.devlineFind) }
                    .keyboardShortcut("f")
                Button("Show Activity") { announce(.devlineActivity) }
            }
        }

        Settings {
            SettingsView(model: model).preferredAppearance()
        }

        MenuBarExtra {
            DevlineStatusMenu()
        } label: {
            Self.menuBarIcon
                .accessibilityLabel("Devline")
        }
        .menuBarExtraStyle(.menu)
    }

    private static let menuBarIcon: Image = {
        guard let url = Bundle.devlineResources.url(forResource: "MenuBarMonogram", withExtension: "png"),
              let image = NSImage(contentsOf: url) else {
            return Image(systemName: "d.square")
        }
        image.size = NSSize(width: 18, height: 18)
        image.isTemplate = true
        image.accessibilityDescription = "Devline"
        return Image(nsImage: image)
    }()

    private func announce(_ action: Notification.Name) {
        NotificationCenter.default.post(name: action, object: nil)
    }

    private func show(_ page: String) {
        NotificationCenter.default.post(name: .devlineShowPage, object: page)
    }
}

private struct DevlineStatusMenu: View {
    @Environment(\.openWindow) private var openWindow

    var body: some View {
        Button("Open Devline") {
            NSApp.unhide(nil)
            openWindow(id: "workspace")
            NSApp.activate()
        }
        Divider()
        Button("Quit Devline") { NSApp.terminate(nil) }
    }
}

/// The Open Recent menu. It follows the saved list on its own, so the scenes are not rebuilt
/// when a project is opened.
private struct OpenRecentItems: View {
    let model: WorkspaceModel
    @ObservedObject private var saved = PreferenceBox.box(RecentProject.storageKey)

    var body: some View {
        let recentProjects = RecentProject.restore(UserDefaults.standard.data(forKey: saved.key))
        if recentProjects.isEmpty { Text("No Recent Projects") }
        ForEach(recentProjects) { project in
            Button(project.name) { NotificationCenter.default.post(name: .devlineOpenRecent, object: project.path) }
                .help(project.path)
        }
        if !recentProjects.isEmpty {
            Divider()
            Button("Clear Menu") { model.clearRecentProjects() }
        }
    }
}
