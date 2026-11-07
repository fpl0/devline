import SwiftUI

struct RuntimesView: View {
    @ObservedObject var model: WorkspaceModel
    let projectStack: String?
    let showStarter: (String) -> Void
    let showDocs: (String) -> Void
    let showLibrary: () -> Void

    @State private var runtimes: [RuntimeToolchain] = []
    @State private var selection: String?
    @State private var scanRevision = 0
    @State private var scanning = true
    @State private var scanError: String?

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 10) {
                if scanning { ProgressView().controlSize(.small) }
                Text(scanning ? "Checking local toolchains…" : "\(runtimes.filter(\.available).count) of \(runtimes.count) ready on this Mac")
                    .font(.callout).foregroundStyle(.secondary)
                Spacer()
                Button {
                    scanRevision += 1
                } label: {
                    Label("Rescan", systemImage: "arrow.clockwise")
                }
                .disabled(scanning)
                .help("Check local toolchains again")
            }
            .padding(.horizontal, 18)
            .padding(.vertical, 13)
            Divider()
            if let scanError, !runtimes.isEmpty {
                Label(scanError, systemImage: "exclamationmark.circle")
                    .font(.callout).foregroundStyle(.secondary)
                    .padding(.horizontal, 18).padding(.vertical, 8)
                    .frame(maxWidth: .infinity, alignment: .leading)
                Divider()
            }
            if let scanError, runtimes.isEmpty {
                ContentUnavailableView("Could Not Scan", systemImage: "exclamationmark.circle", description: Text(scanError))
            } else if runtimes.isEmpty, scanning {
                ProgressView().frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if runtimes.isEmpty {
                ContentUnavailableView("No Runtimes", systemImage: "terminal", description: Text("No runtime toolchains are listed in the catalog."))
            } else {
                HSplitView {
                    List(runtimes, selection: $selection) { runtime in
                        HStack(spacing: 9) {
                            VStack(alignment: .leading, spacing: 3) {
                                Text(runtime.name).font(.body.weight(.medium))
                                Text(runtime.available ? "Detected" : "Setup Required")
                                    .font(.caption).foregroundStyle(.secondary)
                            }
                            Spacer(minLength: 0)
                            Image(systemName: runtime.available ? "checkmark.circle" : "exclamationmark.circle")
                                .foregroundStyle(.secondary)
                                .accessibilityHidden(true)
                        }
                        .padding(.vertical, 5)
                        .tag(runtime.id)
                    }
                    .listStyle(.inset)
                    .frame(minWidth: 208, idealWidth: 238, maxWidth: 305)
                    Group {
                        if let runtime = runtimes.first(where: { $0.id == selection }) {
                            runtimeDetails(runtime).id(runtime.id)
                        } else {
                            ContentUnavailableView("Select a Runtime", systemImage: "terminal")
                        }
                    }
                    .frame(minWidth: 350, maxWidth: .infinity, maxHeight: .infinity)
                }
            }
        }
        .task(id: scanRevision) { await scan() }
    }

    private func runtimeDetails(_ runtime: RuntimeToolchain) -> some View {
        let packs = docPacks(for: runtime)
        let starters = latestStarters(for: runtime.id)
        return ScrollView {
            VStack(alignment: .leading, spacing: 18) {
                VStack(alignment: .leading, spacing: 7) {
                    if projectStack == runtime.id {
                        Label("Current Project", systemImage: "folder")
                            .font(.caption.weight(.semibold)).foregroundStyle(.secondary)
                    }
                    Text(runtime.name).font(.title2.weight(.semibold))
                    HStack(spacing: 7) {
                        Image(systemName: runtime.available ? "checkmark.circle.fill" : "exclamationmark.circle.fill")
                            .foregroundStyle(runtime.available ? Color.green : .orange)
                            .accessibilityHidden(true)
                        Text(runtime.available ? "Compatible Toolchain Detected" : "Setup Required")
                            .font(.callout.weight(.medium))
                    }
                    if runtime.available {
                        Text(detectedVersion(runtime))
                            .font(.callout.monospacedDigit()).foregroundStyle(.secondary)
                            .lineLimit(2)
                    } else if !runtime.minimumVersion.isEmpty {
                        Text("Requires \(runtime.toolchain) \(runtime.minimumVersion) to create projects.")
                            .font(.callout).foregroundStyle(.secondary)
                    }
                }
                Divider()
                if !packs.isEmpty { docsRow(packs, for: runtime) }
                if !starters.isEmpty {
                    VStack(alignment: .leading, spacing: 16) {
                        Text("Offline Kits and Starters").font(.headline)
                        ForEach(starters, id: \.reference) { starter in starterRow(starter) }
                    }
                }
                if packs.isEmpty && starters.isEmpty {
                    Button("Open Library", action: showLibrary)
                }
                Divider()
                DisclosureGroup("Toolchain Details") {
                    VStack(alignment: .leading, spacing: 7) {
                        Text("Executable: \(runtime.toolchain)")
                        if !runtime.minimumVersion.isEmpty {
                            Text("Required version: \(runtime.minimumVersion)")
                        }
                        if let path = runtime.path {
                            Text(path).font(.system(.caption, design: .monospaced))
                        }
                        Text(probeOutput(runtime))
                        Text("Discovery checks PATH, standard Homebrew locations, mise installations, and Rust toolchain directories. Devline runs version probes and compares them with starter requirements. Java/Maven and Elixir/Erlang are checked together.")
                    }
                    .font(.callout).foregroundStyle(.secondary)
                    .textSelection(.enabled)
                    .fixedSize(horizontal: false, vertical: true)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(.top, 7)
                }
                .font(.callout)
                Text("Devline detects your installed tools. It does not install, upgrade, or remove runtimes. Use your preferred runtime manager to change them.")
                    .font(.caption).foregroundStyle(.secondary)
            }
            .frame(maxWidth: 580, alignment: .leading)
            .padding(26)
            .frame(maxWidth: .infinity, alignment: .topLeading)
        }
    }

    private func docsRow(_ packs: [DocPack], for runtime: RuntimeToolchain) -> some View {
        let missing = packs.filter { !$0.installed }
        return HStack(alignment: .center, spacing: 12) {
            VStack(alignment: .leading, spacing: 4) {
                Text("Documentation").font(.headline)
                Text(packs.map(\.name).joined(separator: ", "))
                    .font(.callout).foregroundStyle(.secondary)
            }
            Spacer(minLength: 0)
            if missing.count == packs.count {
                Button("Install Docs") { installDocs(missing, for: runtime) }
                    .disabled(model.operation != nil)
            } else {
                Button("Browse Docs") { showDocs(runtime.id) }
                if !missing.isEmpty {
                    Menu {
                        Button("Install Missing Docs") { installDocs(missing, for: runtime) }
                    } label: {
                        Label("More Documentation Actions", systemImage: "ellipsis.circle")
                            .labelStyle(.iconOnly)
                    }
                    .menuStyle(.borderlessButton)
                    .fixedSize()
                    .accessibilityLabel("More Documentation Actions")
                    .disabled(model.operation != nil)
                }
            }
        }
    }

    private func starterRow(_ starter: Starter) -> some View {
        HStack(alignment: .center, spacing: 12) {
            VStack(alignment: .leading, spacing: 4) {
                Text("\(starter.name) · \(starter.version)").font(.callout.weight(.medium))
                Text(starter.description).font(.caption).foregroundStyle(.secondary).fixedSize(horizontal: false, vertical: true)
            }
            Spacer(minLength: 0)
            if starter.isInstalled {
                Button("View Starter") { showStarter(starter.reference) }
            } else {
                Button("Prepare Starter") { prepare(starter) }
                    .buttonStyle(.borderedProminent)
                    .disabled(model.operation != nil)
                    .help("Install the starter and related docs from the bundled catalog")
            }
        }
    }

    private func latestStarters(for stack: String) -> [Starter] {
        let candidates: [Starter] = (model.catalog?.templates ?? []).filter { $0.stack == stack && $0.verified }
        let families: [String: [Starter]] = Dictionary(grouping: candidates, by: \.id)
        var latest: [Starter] = []
        for releases in families.values {
            if let starter = releases.max(by: { $0.version.compare($1.version, options: .numeric) == .orderedAscending }) { latest.append(starter) }
        }
        return latest.sorted { $0.name.localizedStandardCompare($1.name) == .orderedAscending }
    }

    private func docPacks(for runtime: RuntimeToolchain) -> [DocPack] {
        let packs = model.catalog?.docs ?? []
        return runtime.docs.compactMap { reference in
            packs.first { $0.reference == reference }
                ?? packs.filter { $0.id == reference }.max {
                    $0.version.compare($1.version, options: .numeric) == .orderedAscending
                }
        }
    }

    private func probeOutput(_ runtime: RuntimeToolchain) -> String {
        guard let path = runtime.path else { return runtime.detail }
        let prefix = "\(path) · "
        return runtime.detail.hasPrefix(prefix) ? String(runtime.detail.dropFirst(prefix.count)) : runtime.detail
    }

    private func detectedVersion(_ runtime: RuntimeToolchain) -> String {
        let output = probeOutput(runtime)
        func version(after marker: String = "", allowMajorOnly: Bool = false) -> String? {
            let suffix: Substring
            if marker.isEmpty {
                suffix = output[...]
            } else if let range = output.range(of: marker) {
                suffix = output[range.upperBound...]
            } else {
                return nil
            }
            let pattern = allowMajorOnly ? #"[0-9]+(?:\.[0-9]+)*"# : #"[0-9]+(?:\.[0-9]+)+"#
            return suffix.range(of: pattern, options: .regularExpression).map { String(suffix[$0]) }
        }
        switch runtime.toolchain {
        case "mvn":
            if let maven = version(after: "Apache Maven "), let java = version(after: "Java version: ", allowMajorOnly: true) {
                return "Maven \(maven) · Java \(java)"
            }
        case "elixir":
            if let elixir = version(after: "Elixir "), let otp = version(after: "Erlang/OTP ", allowMajorOnly: true) {
                return "Elixir \(elixir) · Erlang/OTP \(otp)"
            }
        case "python3":
            if let python = version() { return "CPython \(python)" }
        default:
            if let detected = version() { return "Version \(detected)" }
        }
        return output
    }

    private func scan() async {
        scanning = true
        scanError = nil
        defer { scanning = false }
        do {
            runtimes = try await Core.request(CoreRequest("runtime.list"))
            if !runtimes.contains(where: { $0.id == selection }) {
                selection = runtimes.first(where: { $0.id == projectStack })?.id ?? runtimes.first?.id
            }
        } catch is CancellationError {
        } catch {
            scanError = error.localizedDescription
        }
    }

    private func installDocs(_ packs: [DocPack], for runtime: RuntimeToolchain) {
        let requests = packs.map { CoreRequest("docs.install", id: $0.id, version: $0.version) }
        Task {
            let installed = await model.mutate(requests, label: "Installing \(runtime.name) docs",
                                               completion: "Installed \(runtime.name) documentation.")
            if installed { showDocs(runtime.id) }
        }
    }

    private func prepare(_ starter: Starter) {
        Task {
            await model.startJob(CoreRequest("jobs.start", id: starter.reference, kind: "prepare"))
        }
    }
}
