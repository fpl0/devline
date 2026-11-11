import AppKit
import SwiftUI

private struct ProjectCommands: Decodable, Equatable, Sendable {
    let path: String
    let launchable: Bool
    let runCommand, testCommand, guidePath, detail: String?
}

private struct ProjectAppStatus: Decodable, Equatable, Sendable {
    let path, state: String
    let running: Bool
    let pid: Int?
    let url, detail: String?

    var starting: Bool { state == "starting" }
    var failed: Bool { state == "failed" }
    var localURL: URL? {
        guard let url, let value = safeExternalURL(url),
              ["localhost", "127.0.0.1", "::1", "[::1]"].contains(value.host?.lowercased() ?? "") else { return nil }
        return value
    }
}

/// The last run state read for each project, so its page is complete the moment it appears.
@MainActor private var knownRuns: [String: (commands: ProjectCommands?, status: ProjectAppStatus?, log: String)] = [:]

/// Runs the project's app with the network blocked and shows its output as it arrives.
struct ProjectRunView: View {
    let path: String
    let preparationRevision: [String]
    var visible = true
    let refreshServices: () -> Void
    @Environment(\.scenePhase) private var scenePhase
    @State private var freshCommands: ProjectCommands?
    @State private var freshStatus: ProjectAppStatus?
    @State private var freshLog: String?
    private var commands: ProjectCommands? { freshCommands ?? knownRuns[path]?.commands }
    private var status: ProjectAppStatus? { freshStatus ?? knownRuns[path]?.status }
    private var log: String { freshLog ?? knownRuns[path]?.log ?? "" }
    @State private var guideURL: URL?
    @State private var changing = false
    @State private var error: String?
    @State private var expanded = false
    @State private var copied: String?
    @State private var showCommands = false

    private struct CommandRefresh: Equatable {
        let preparation: [String]
        let phase: ScenePhase
    }

    private struct Watching: Equatable {
        let phase: ScenePhase
        let visible: Bool
    }

    private var active: Bool { status?.running == true || status?.starting == true }
    private var starting: Bool { (changing && status?.running != true) || status?.starting == true }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack(spacing: 10) {
                if status?.running == true {
                    Button { Task { await change(start: false) } } label: {
                        Label("Stop", systemImage: "stop.fill").frame(minWidth: 52)
                    }.controlSize(.large).disabled(changing)
                } else {
                    Button { Task { await change(start: true) } } label: {
                        Label(starting ? "Starting…" : "Run", systemImage: "play.fill").frame(minWidth: 52)
                    }
                    .buttonStyle(.borderedProminent).controlSize(.large)
                    .disabled(changing || commands?.launchable != true)
                    .help("Run with internet access blocked")
                }
                if let url = status?.localURL, status?.running == true {
                    Button { NSWorkspace.shared.open(url) } label: {
                        Label("Open App", systemImage: "arrow.up.forward.square")
                    }.controlSize(.large).disabled(status?.state != "running")
                }
                statusLabel
                Spacer(minLength: 8)
                if guideURL != nil, let guide = commands?.guidePath {
                    Button("Project Guide") { openGuide(guide) }.help("Open the project's README")
                }
            }
            if commands?.launchable == false {
                Text(commands?.detail ?? "This project has no run command yet. Its guide explains how to start it, or run a command from the terminal.")
                    .font(.callout).foregroundStyle(.secondary).textSelection(.enabled)
            } else if let detail = status?.detail, !detail.isEmpty, status?.failed == true {
                Label(detail, systemImage: "exclamationmark.circle").font(.callout).foregroundStyle(.orange).textSelection(.enabled)
            }
            if !log.isEmpty || active || changing {
                console(height: 190)
            } else if commands?.launchable == true {
                Text("Runs with internet access blocked, so what works here works offline. Its configured services start from their saved images.")
                    .font(.caption).foregroundStyle(.secondary)
            }
            if commands?.runCommand != nil || commands?.testCommand != nil {
                DisclosureGroup("Terminal commands", isExpanded: $showCommands) {
                    VStack(alignment: .leading, spacing: 10) {
                        if let command = commands?.runCommand { commandRow("Run", command: command) }
                        if let command = commands?.testCommand { commandRow("Test", command: command) }
                    }.padding(.top, 8)
                }.font(.callout)
            }
            if let error {
                Label(error, systemImage: "exclamationmark.circle").font(.callout).foregroundStyle(.orange).textSelection(.enabled)
            }
        }
        .onChange(of: status?.running) { previous, _ in
            if previous != nil { refreshServices() }
        }
        .task(id: CommandRefresh(preparation: preparationRevision, phase: scenePhase)) {
            if scenePhase == .active { await loadCommands() }
        }
        .task(id: Watching(phase: scenePhase, visible: visible)) {
            // A project page that is loaded but not on screen checks once, then waits to be shown.
            guard scenePhase == .active else { return }
            guard visible else {
                if status == nil { await refreshStatus() }
                return
            }
            var wasActive = true
            while !Task.isCancelled {
                if !changing { await refreshStatus() }
                // Follow output while the app runs, and once more after it stops.
                if active || wasActive { await loadLog() }
                wasActive = active
                do { try await Task.sleep(for: active ? .seconds(1) : .seconds(2)) } catch { return }
            }
        }
        .sheet(isPresented: $expanded) {
            VStack(alignment: .leading, spacing: 12) {
                HStack {
                    Text("App Output").font(.title2.weight(.semibold))
                    statusLabel
                    Spacer()
                    Button("Done") { expanded = false }.keyboardShortcut(.cancelAction)
                }
                console(height: nil, expandable: false)
            }.padding(20).frame(minWidth: 820, idealWidth: 900, minHeight: 520, idealHeight: 620)
        }
    }

    @ViewBuilder private var statusLabel: some View {
        if let status {
            HStack(spacing: 6) {
                if starting { ProgressView().controlSize(.small) }
                else {
                    Circle().fill(status.running ? Color.green : status.failed ? .orange : Color.secondary.opacity(0.5))
                        .frame(width: 8, height: 8)
                }
                Text(starting ? "Starting" : status.running ? runningSummary(status) : status.failed ? "Stopped with an error" : "Not running")
                    .font(.callout).foregroundStyle(.secondary).lineLimit(1)
            }
            .accessibilityElement(children: .combine)
        }
    }

    private func runningSummary(_ status: ProjectAppStatus) -> String {
        guard let url = status.localURL, let host = url.host else { return "Running" }
        return "Running at \(host)\(url.port.map { ":\($0)" } ?? "")"
    }

    private func console(height: CGFloat?, expandable: Bool = true) -> some View {
        ScrollViewReader { proxy in
            ScrollView {
                Text(log.isEmpty ? "Waiting for output…" : log)
                    .font(.system(size: 11.5, design: .monospaced))
                    .foregroundStyle(log.isEmpty ? .secondary : .primary)
                    .textSelection(.enabled)
                    .frame(maxWidth: .infinity, alignment: .topLeading)
                    .padding(12)
                Color.clear.frame(height: 1).id("end")
            }
            .defaultScrollAnchor(.bottom)
            .onChange(of: log) { _, _ in proxy.scrollTo("end", anchor: .bottom) }
        }
        .frame(height: height)
        .frame(maxHeight: height == nil ? .infinity : nil)
        .background(Color(nsColor: .textBackgroundColor).opacity(0.6), in: RoundedRectangle(cornerRadius: 8))
        .overlay { RoundedRectangle(cornerRadius: 8).strokeBorder(Color.primary.opacity(0.08)) }
        .overlay(alignment: .topTrailing) {
            HStack(spacing: 2) {
                Button { if copyToPasteboard(log) { copied = "log" } } label: {
                    Image(systemName: copied == "log" ? "checkmark" : "doc.on.doc")
                }.help("Copy output").accessibilityLabel("Copy output")
                if expandable {
                    Button { expanded = true } label: { Image(systemName: "arrow.up.left.and.arrow.down.right") }
                        .help("Open output in a larger view").accessibilityLabel("Expand output")
                }
            }
            .buttonStyle(.borderless).padding(6)
            .background(.regularMaterial, in: RoundedRectangle(cornerRadius: 6)).padding(6)
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
            let guide: URL?
            if let guidePath = next.guidePath {
                guide = await Task.detached { [path] in Self.resolveGuide(guidePath, in: path) }.value
            } else {
                guide = nil
            }
            guard !Task.isCancelled else { return }
            if guideURL != guide { guideURL = guide }
            if commands != next { freshCommands = next }
            knownRuns[path, default: (nil, nil, "")].commands = next
        } catch is CancellationError {} catch { self.error = error.localizedDescription }
    }

    private func refreshStatus() async {
        do {
            let next: ProjectAppStatus = try await Core.request(CoreRequest("project.status", path: path))
            guard !Task.isCancelled else { return }
            if status != next { freshStatus = next }
            knownRuns[path, default: (nil, nil, "")].status = next
        } catch is CancellationError {} catch { self.error = error.localizedDescription }
    }

    private func change(start: Bool) async {
        guard !changing else { return }
        changing = true; error = nil
        if start { freshLog = ""; knownRuns[path, default: (nil, nil, "")].log = "" }
        let wasRunning = status?.running
        defer { changing = false }
        do {
            let next: ProjectAppStatus = try await Core.request(CoreRequest(start ? "project.start" : "project.stop", path: path, offline: true))
            freshStatus = next
            knownRuns[path, default: (nil, nil, "")].status = next
        } catch { self.error = error.localizedDescription; await refreshStatus() }
        await loadLog()
        if wasRunning == nil || status?.running == wasRunning { refreshServices() }
    }

    private func loadLog() async {
        struct Output: Decodable { let log: String? }
        guard let output: Output = try? await Core.request(CoreRequest("project.logs", path: path, tail: 400)),
              !Task.isCancelled else { return }
        let next = output.log ?? ""
        if log != next {
            freshLog = next
            knownRuns[path, default: (nil, nil, "")].log = next
            if copied == "log" { copied = nil }
        }
    }

    private nonisolated static func resolveGuide(_ guide: String, in path: String) -> URL? {
        let root = URL(fileURLWithPath: path).resolvingSymlinksInPath().standardizedFileURL.path
        let url = URL(fileURLWithPath: guide).resolvingSymlinksInPath().standardizedFileURL
        return url.path.hasPrefix(root + "/") && FileManager.default.fileExists(atPath: url.path) ? url : nil
    }

    private func openGuide(_ guide: String) {
        guard let url = Self.resolveGuide(guide, in: path) else {
            error = "The project guide is no longer available."
            return
        }
        NSWorkspace.shared.open(url)
    }
}
