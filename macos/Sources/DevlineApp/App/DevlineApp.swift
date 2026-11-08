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
}

@main
struct DevlineMacApp: App {
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
                    Button("App Updates and Downloads…") { announce(.devlineAppUpdates) }
                }
                Button("Library Updates…") { announce(.devlineUpdates) }
            }
            CommandGroup(replacing: .newItem) {
                Button("New from Starter…") { announce(.devlineNew) }
                    .keyboardShortcut("n")
                Button("Open Project Folder…") { announce(.devlineOpenProject) }
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
                Button("Find in Docs") { announce(.devlineFind) }
                    .keyboardShortcut("f")
                Button("Open Offline Library") { announce(.devlineLibrary) }
                    .keyboardShortcut("l", modifiers: [.command, .shift])
                Button("Show Activity") { announce(.devlineActivity) }
            }
        }

        Settings {
            SettingsView(model: model)
                .preferredColorScheme(appearance == "system" ? nil : appearance == "light" ? .light : .dark)
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
