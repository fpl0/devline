import AppKit
import SwiftUI

struct SettingsView: View {
    @ObservedObject var model: WorkspaceModel
    @Preference("devline.appearance") private var appearance = "system"
    @State private var cliLocation: String?
    @State private var cliError: String?
    @State private var pathCopied = false
    @State private var retention: LibraryRetention?
    @State private var retentionBusy = false
    @State private var purging = false
    @State private var confirmPurge = false
    @State private var libraryNotice: String?
    @State private var libraryError: String?

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
                if let retention {
                    Toggle("Keep only the latest version after updates", isOn: Binding(
                        get: { retention.keepLatest },
                        set: { enabled in Task { await loadRetention(keepLatest: enabled) } }
                    ))
                    .disabled(retentionBusy || purging)
                }
                Text("When Devline installs the newest local release of a docset, starter, or image archive, it purges older releases of that item.")
                    .font(.caption).foregroundStyle(.secondary)
                Button("Purge Older Versions…") { confirmPurge = true }
                    .disabled(retentionBusy || model.operation != nil || model.jobs.contains(where: \.isActive))
                if purging {
                    ProgressView("Purging older versions…").controlSize(.small)
                }
                if let libraryNotice {
                    Text(libraryNotice).font(.caption).foregroundStyle(.secondary)
                }
                if let libraryError {
                    Label(libraryError, systemImage: "exclamationmark.circle")
                        .font(.callout).foregroundStyle(.red)
                }
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
        .task { await loadRetention() }
        .alert("Purge older library versions?", isPresented: $confirmPurge) {
            Button("Purge Older Versions", role: .destructive) { Task { await purgeOldVersions() } }
            Button("Cancel", role: .cancel) {}
        } message: {
            Text("Devline will keep the newest local release of each docset, starter, and image archive. Projects pinned to a removed release may need that release prepared again.")
        }
    }

    private func loadRetention(keepLatest: Bool? = nil) async {
        retentionBusy = true
        libraryError = nil
        defer { retentionBusy = false }
        do {
            retention = try await Core.request(CoreRequest("library.retention", keepLatest: keepLatest))
        } catch is CancellationError {
        } catch {
            libraryError = error.localizedDescription
        }
    }

    private func purgeOldVersions() async {
        guard model.operation == nil else { return }
        purging = true
        defer { purging = false }
        libraryNotice = nil
        libraryError = nil
        do {
            libraryNotice = try await model.purgeOldVersions().summary
        } catch {
            libraryError = error.localizedDescription
        }
    }

    private func installCLI() {
        cliLocation = nil
        cliError = nil
        do {
            cliLocation = try installDevlineCommand().path(percentEncoded: false)
        } catch {
            cliError = error.localizedDescription
        }
    }

    private func copyPathCommand() {
        pathCopied = copyToPasteboard(#"export PATH="$HOME/.local/bin:$PATH""#)
    }
}

/// Where the `devline` command is linked for Terminal and for projects' `devline.sh`.
var devlineCommandLocation: URL {
    FileManager.default.homeDirectoryForCurrentUser.appendingPathComponent(".local/bin/devline")
}

/// Links the command bundled with this app into `~/.local/bin`.
@discardableResult
func installDevlineCommand() throws -> URL {
    guard let executable = Bundle.main.executableURL else {
        throw CoreError(message: "Open the packaged Devline.app to install the CLI.")
    }
    return try installDevlineLink(target: executable.deletingLastPathComponent().appendingPathComponent("devline"),
                                  directory: devlineCommandLocation.deletingLastPathComponent())
}

private func installDevlineLink(target: URL, directory: URL) throws -> URL {
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
