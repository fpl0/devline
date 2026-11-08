import AppKit
import SwiftUI

private struct ProjectCommands: Decodable, Sendable {
    let path: String
    let launchable: Bool
    let runCommand, testCommand, guidePath, detail: String?
}

private struct ProjectAppStatus: Decodable, Sendable {
    let path, state: String
    let running: Bool
    let pid: Int?
    let url, detail: String?

    var localURL: URL? {
        guard let url, let value = safeExternalURL(url),
              ["localhost", "127.0.0.1", "::1", "[::1]"].contains(value.host?.lowercased() ?? "") else { return nil }
        return value
    }
}

struct ProjectRunView: View {
    let path: String
    let preparationRevision: [String]
    let refreshServices: () -> Void
    @Environment(\.scenePhase) private var scenePhase
    @State private var commands: ProjectCommands?
    @State private var status: ProjectAppStatus?
    @State private var changing = false
    @State private var error: String?
    @State private var log: String?
    @State private var copied: String?
    @State private var showDetails = false

    private var editors: [URL] {
        ["com.microsoft.VSCode", "com.todesktop.230313mzl4w4u92", "dev.zed.Zed", "com.jetbrains.rider", "com.apple.dt.Xcode"]
            .compactMap { NSWorkspace.shared.urlForApplication(withBundleIdentifier: $0) }
    }
    private var guideURL: URL? {
        guard let guide = commands?.guidePath else { return nil }
        let root = URL(fileURLWithPath: path).resolvingSymlinksInPath().standardizedFileURL.path
        let url = URL(fileURLWithPath: guide).resolvingSymlinksInPath().standardizedFileURL
        guard url.path.hasPrefix(root + "/"), FileManager.default.fileExists(atPath: url.path) else { return nil }
        return url
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            HStack(alignment: .top, spacing: 14) {
                VStack(alignment: .leading, spacing: 5) {
                    Label("Develop & Run", systemImage: "play.circle").font(.title3.weight(.semibold))
                    Text("Edit your project, then run it with its local dependencies and services.")
                        .font(.callout).foregroundStyle(.secondary)
                }
                Spacer()
                if editors.isEmpty {
                    Button("Open Project Folder") { NSWorkspace.shared.open(URL(fileURLWithPath: path)) }
                } else {
                    Menu("Open in Editor") {
                        ForEach(editors, id: \.path) { editor in
                            Button(editor.deletingPathExtension().lastPathComponent) { openEditor(editor) }
                        }
                        Divider()
                        Button("Open Project Folder") { NSWorkspace.shared.open(URL(fileURLWithPath: path)) }
                    }.fixedSize()
                }
                if let guideURL { Button("Read Project Guide") { NSWorkspace.shared.open(guideURL) } }
                Button { Task { await loadCommands(); await refreshStatus() } } label: { Image(systemName: "arrow.clockwise") }
                    .help("Refresh run commands and app status").accessibilityLabel("Refresh app status").disabled(changing)
            }
            HStack(spacing: 10) {
                Button { Task { await change(start: true) } } label: {
                    Label((changing && status?.running != true) || status?.state == "starting" ? "Starting…" : "Run Offline", systemImage: "play.fill")
                }.buttonStyle(.borderedProminent).disabled(changing || commands?.launchable != true || status?.running == true)
                if status?.running == true {
                    Button("Stop App") { Task { await change(start: false) } }.disabled(changing)
                    if let url = status?.localURL { Button("Open App") { NSWorkspace.shared.open(url) }.disabled(status?.state != "running") }
                }
                Button("App Logs") { Task { await loadLog() } }.disabled(status == nil)
                if changing || status?.state == "starting" { ProgressView().controlSize(.small) }
                Spacer()
                if let status {
                    Label(status.state == "starting" ? "Starting locally" : status.running ? "Running locally" : status.state == "failed" ? "App needs attention" : "Stopped",
                          systemImage: status.state == "running" ? "circle.fill" : status.state == "failed" ? "exclamationmark.circle" : "circle")
                        .font(.caption).foregroundStyle(status.state == "running" ? Color.green : status.state == "failed" ? .orange : .secondary)
                }
            }
            if commands?.launchable == false {
                Text(commands?.detail ?? "This project does not have a run command yet. Read its guide or use the CLI to run an explicit command.")
                    .font(.callout).foregroundStyle(.secondary).textSelection(.enabled)
            } else if let detail = status?.detail, !detail.isEmpty {
                Text(detail).font(.caption).foregroundStyle(.secondary).textSelection(.enabled)
            }
            Text("Run Offline starts configured services from their saved archives. Stopping the app keeps services and their data.")
                .font(.caption).foregroundStyle(.secondary)
            if commands?.runCommand != nil || commands?.testCommand != nil {
                DisclosureGroup("Terminal commands", isExpanded: $showDetails) {
                    VStack(alignment: .leading, spacing: 10) {
                        if let command = commands?.runCommand { commandRow("Run", command: command) }
                        if let command = commands?.testCommand { commandRow("Test", command: command) }
                    }.padding(.top, 8)
                }.font(.callout)
            }
            if let error {
                Label(error, systemImage: "exclamationmark.circle").font(.callout).foregroundStyle(.orange).textSelection(.enabled)
            }
        }.padding(18).background(.quaternary.opacity(0.35), in: RoundedRectangle(cornerRadius: 12))
        .onChange(of: status?.running) { _, _ in refreshServices() }
        .task(id: preparationRevision) { await loadCommands() }
        .task(id: scenePhase) {
            guard scenePhase == .active else { return }
            await loadCommands()
            while !Task.isCancelled {
                if !changing { await refreshStatus() }
                do { try await Task.sleep(for: .seconds(2)) } catch { return }
            }
        }
        .sheet(isPresented: Binding(get: { log != nil }, set: { if !$0 { log = nil } })) {
            VStack(alignment: .leading, spacing: 14) {
                HStack {
                    Text("App Logs").font(.title2.weight(.semibold))
                    Spacer()
                    Button("Refresh") { Task { await loadLog() } }
                    Button("Done") { log = nil }.keyboardShortcut(.cancelAction)
                }
                Text((path as NSString).abbreviatingWithTildeInPath).font(.caption).foregroundStyle(.secondary)
                ScrollView([.vertical, .horizontal]) {
                    Text(log.flatMap { $0.isEmpty ? nil : $0 } ?? "No app output yet. Run the app to see its output here.")
                        .font(.system(.caption, design: .monospaced)).textSelection(.enabled)
                        .frame(maxWidth: .infinity, alignment: .topLeading)
                }
            }.padding(22).frame(width: 760, height: 480)
        }
    }

    private func commandRow(_ title: String, command: String) -> some View {
        HStack(alignment: .top, spacing: 12) {
            VStack(alignment: .leading, spacing: 4) {
                Text(title).font(.caption.weight(.semibold))
                Text(command).font(.system(.caption, design: .monospaced)).textSelection(.enabled)
                    .fixedSize(horizontal: false, vertical: true)
            }
            Spacer()
            Button(copied == title ? "Copied" : "Copy") {
                if copyToPasteboard(command) { copied = title }
            }.controlSize(.small).accessibilityLabel("Copy \(title.lowercased()) command")
        }
    }
    private func loadCommands() async {
        do {
            let next: ProjectCommands = try await Core.request(CoreRequest("project.commands", path: path))
            guard !Task.isCancelled else { return }
            commands = next
        } catch is CancellationError {} catch { self.error = error.localizedDescription }
    }
    private func refreshStatus() async {
        do {
            let next: ProjectAppStatus = try await Core.request(CoreRequest("project.status", path: path))
            guard !Task.isCancelled else { return }
            status = next
        } catch is CancellationError {} catch { self.error = error.localizedDescription }
    }
    private func change(start: Bool) async {
        guard !changing else { return }
        changing = true; error = nil
        let wasRunning = status?.running
        defer { changing = false }
        do {
            let next: ProjectAppStatus = try await Core.request(CoreRequest(start ? "project.start" : "project.stop", path: path, offline: true))
            status = next
        } catch { self.error = error.localizedDescription; await refreshStatus() }
        if status?.running == wasRunning { refreshServices() }
    }
    private func loadLog() async {
        struct Output: Decodable { let log: String? }
        do {
            let output: Output = try await Core.request(CoreRequest("project.logs", path: path, tail: 200))
            log = output.log ?? ""
        } catch { self.error = error.localizedDescription }
    }
    private func openEditor(_ editor: URL) {
        NSWorkspace.shared.open([URL(fileURLWithPath: path)], withApplicationAt: editor, configuration: NSWorkspace.OpenConfiguration()) { _, failure in
            if let failure { Task { @MainActor in error = failure.localizedDescription } }
        }
    }
}
