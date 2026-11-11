import SwiftUI

@MainActor
final class RuntimeInventory: ObservableObject {
    struct Snapshot: Decodable, Equatable, Sendable {
        let runtimes: [RuntimeToolchain]
        let releases: [UpdateItem]
    }

    @Published private(set) var snapshot: Snapshot?
    @Published private(set) var scanning = true
    @Published private(set) var error: String?
    private var loadTask: Task<Void, Never>?
    private var lastLoaded: ContinuousClock.Instant?

    func load(force: Bool) async {
        if let loadTask { return await loadTask.value }
        if !force, let lastLoaded, lastLoaded.duration(to: .now) < .seconds(3) { return }
        // A refresh behind what is already on screen stays quiet.
        // Only what changed is announced, so the pages that are loaded are not rebuilt for nothing.
        if scanning != (force || snapshot == nil) { scanning.toggle() }
        if error != nil { error = nil }
        let task = Task {
            defer {
                if scanning { scanning = false }
                loadTask = nil
            }
            do {
                let next: Snapshot = try await Core.request(CoreRequest("runtime.inventory"))
                if snapshot != next { snapshot = next }
                lastLoaded = .now
            } catch is CancellationError {
            } catch {
                self.error = error.localizedDescription
            }
        }
        loadTask = task
        await task.value
    }
}

struct RuntimesView: View {
    @ObservedObject var model: WorkspaceModel
    @ObservedObject var inventory: RuntimeInventory
    @ObservedObject var infrastructure: InfrastructureLibrary
    /// A runtime id, or `docker` for the container engine.
    @Binding var selection: String?
    let projectStack: String?
    var visible = true
    let showStarter: (String) -> Void
    let showDocs: (String) -> Void
    let showLibrary: () -> Void

    @State private var scanRevision = 0
    @State private var copied: String?
    private var runtimes: [RuntimeToolchain] { inventory.snapshot?.runtimes ?? [] }
    private var releases: [UpdateItem] { inventory.snapshot?.releases ?? [] }
    private var scanning: Bool { inventory.scanning }
    private var scanError: String? { inventory.error }

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
                    List(selection: $selection) {
                        ForEach(runtimes) { runtime in
                            row(runtime.name, found: runtime.available).tag(runtime.id)
                        }
                        Section("Services") {
                            row("Docker", found: infrastructure.status?.docker.available == true,
                                state: infrastructure.status == nil ? "Checking…"
                                    : infrastructure.status?.docker.available == true ? "Running" : "Not running").tag("docker")
                        }
                    }
                    .listStyle(.inset)
                    .frame(minWidth: 208, idealWidth: 238, maxWidth: 305)
                    Group {
                        if selection == "docker" {
                            dockerDetails
                        } else if let runtime = runtimes.first(where: { $0.id == selection }) {
                            runtimeDetails(runtime).id(runtime.id)
                        } else {
                            ContentUnavailableView("Select a Runtime", systemImage: "terminal")
                        }
                    }
                    .frame(minWidth: 350, maxWidth: .infinity, maxHeight: .infinity)
                }
            }
        }
        .task(id: scanRevision) { [revision = scanRevision] in
            selectRuntimeIfNeeded()
            async let docker: Void = revision > 0 ? infrastructure.refresh() : infrastructure.loadIfNeeded(after: model.completedInfrastructureJobIDs)
            await inventory.load(force: revision > 0)
            await docker
            if !Task.isCancelled { selectRuntimeIfNeeded() }
        }
        .onPageShown(visible) {
            async let docker: Void = infrastructure.loadIfNeeded(after: model.completedInfrastructureJobIDs)
            await inventory.load(force: false)
            await docker
        }
    }

    private func row(_ name: String, found: Bool, state: String? = nil) -> some View {
        HStack(spacing: 9) {
            VStack(alignment: .leading, spacing: 3) {
                Text(name).font(.body.weight(.medium))
                Text(state ?? (found ? "Found" : "Not installed")).font(.caption).foregroundStyle(.secondary)
            }
            Spacer(minLength: 0)
            Image(systemName: found ? "checkmark.circle" : "arrow.down.circle")
                .foregroundStyle(found ? Color.secondary : .orange).accessibilityHidden(true)
        }
        .padding(.vertical, 5)
    }

    /// A command with a copy button, for pasting into a terminal.
    private func command(_ label: String, _ text: String) -> some View {
        VStack(alignment: .leading, spacing: 5) {
            Text(label).font(.caption.weight(.semibold)).foregroundStyle(.secondary)
            HStack(spacing: 10) {
                Text(text).font(.system(.callout, design: .monospaced)).textSelection(.enabled)
                    .fixedSize(horizontal: false, vertical: true)
                Spacer(minLength: 8)
                Button(copied == text ? "Copied" : "Copy") {
                    if copyToPasteboard(text) { copied = text }
                }.controlSize(.small).accessibilityLabel("Copy command")
            }
            .padding(.horizontal, 10).padding(.vertical, 8)
            .background(Color.primary.opacity(0.06), in: RoundedRectangle(cornerRadius: 7))
        }
    }

    private func installGuide(_ runtime: RuntimeToolchain) -> some View {
        let guide = runtime.install
        let commands = guide?.commands ?? []
        return VStack(alignment: .leading, spacing: 12) {
            Text("Install \(runtime.name)").font(.headline)
            Text("Devline builds and runs projects with the toolchains on this Mac. Install this one while you’re connected, then choose Rescan."
                 + (runtime.minimumVersion.isEmpty ? "" : " The starters need \(runtime.toolchain) \(runtime.minimumVersion) or later."))
                .font(.callout).foregroundStyle(.secondary).fixedSize(horizontal: false, vertical: true)
            ForEach(commands) { item in
                command(item.present ? "With \(item.manager), which this Mac has" : "With \(item.manager)", item.command)
            }
            if !commands.isEmpty, !commands.contains(where: \.present) {
                HStack(spacing: 4) {
                    Text("Neither is installed yet. Get").foregroundStyle(.secondary)
                    Link("Homebrew", destination: URL(string: "https://brew.sh")!)
                    Text("or").foregroundStyle(.secondary)
                    Link("mise", destination: URL(string: "https://mise.jdx.dev")!)
                    Text("first, or use the installer below.").foregroundStyle(.secondary)
                }.font(.callout)
            }
            if let note = guide?.note, !note.isEmpty {
                Text(note).font(.callout).foregroundStyle(.secondary).textSelection(.enabled)
                    .fixedSize(horizontal: false, vertical: true)
            }
            HStack(spacing: 12) {
                Button("Rescan") { scanRevision += 1 }.disabled(scanning)
                if let url = guide?.url.flatMap(safeExternalURL) {
                    Link("Official installer and instructions", destination: url).font(.callout)
                }
            }
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .leading)
        .surface(radius: 10)
    }

    private var dockerDetails: some View {
        let docker = infrastructure.status?.docker
        let running = docker?.available == true
        return ScrollView {
            VStack(alignment: .leading, spacing: 18) {
                VStack(alignment: .leading, spacing: 7) {
                    Text("Docker").font(.title2.weight(.semibold))
                    HStack(spacing: 7) {
                        Image(systemName: running ? "checkmark.circle.fill" : "exclamationmark.circle.fill")
                            .foregroundStyle(running ? Color.green : .orange).accessibilityHidden(true)
                        Text(docker == nil ? "Checking…" : running ? "Engine Running" : "No Engine Running")
                            .font(.callout.weight(.medium))
                    }
                    if let detail = docker?.detail, !detail.isEmpty {
                        Text(detail).font(.callout).foregroundStyle(.secondary).textSelection(.enabled)
                    }
                }
                Text("Devline uses a Docker-compatible engine for two things: saving service images such as PostgreSQL and Redis while you’re connected, and running them from those saved images later. Projects without services don’t need it.")
                    .font(.callout).foregroundStyle(.secondary).fixedSize(horizontal: false, vertical: true)
                if !running {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Install an engine").font(.headline)
                        Text("Any one of these works. Install it while you’re connected, start it, then choose Rescan.")
                            .font(.callout).foregroundStyle(.secondary)
                        command("OrbStack, a light engine for the Mac", "brew install --cask orbstack")
                        command("Docker Desktop", "brew install --cask docker-desktop")
                        command("Colima, command line only", "brew install colima docker docker-compose && colima start")
                        HStack(spacing: 12) {
                            Button("Rescan") { scanRevision += 1 }.disabled(scanning || infrastructure.loading)
                            Link("OrbStack", destination: URL(string: "https://orbstack.dev")!)
                            Link("Docker Desktop", destination: URL(string: "https://www.docker.com/products/docker-desktop/")!)
                            Link("Colima", destination: URL(string: "https://github.com/abiosoft/colima")!)
                        }.font(.callout)
                    }
                    .padding(16)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .surface(radius: 10)
                    Text("Already installed? Open the engine’s app, or run colima start, so Devline can reach it.")
                        .font(.caption).foregroundStyle(.secondary)
                }
            }
            .frame(maxWidth: 580, alignment: .leading)
            .padding(26)
            .frame(maxWidth: .infinity, alignment: .topLeading)
        }
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
                        Text(runtime.available ? "Found on This Mac" : "Not Installed")
                            .font(.callout.weight(.medium))
                    }
                    if runtime.available {
                        Text(detectedVersion(runtime))
                            .font(.callout.monospacedDigit()).foregroundStyle(.secondary)
                            .lineLimit(2)
                    }
                }
                if !runtime.available { installGuide(runtime) }
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
                        Text("Discovery checks PATH, standard Homebrew locations, mise installations, and Rust toolchain directories. Devline runs version probes and compares them with starter requirements. Java/Maven and Elixir/Erlang are checked together. Swift uses the compiler and macOS SDK selected by xcode-select or DEVELOPER_DIR; the SwiftUI starter also requires Xcode’s Swift Testing framework.")
                    }
                    .font(.callout).foregroundStyle(.secondary)
                    .textSelection(.enabled)
                    .fixedSize(horizontal: false, vertical: true)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(.top, 7)
                }
                .font(.callout)
                if !publisherReleases(for: runtime).isEmpty {
                    DisclosureGroup("Publisher Releases") {
                        VStack(alignment: .leading, spacing: 10) {
                            ForEach(publisherReleases(for: runtime)) { release in
                                VStack(alignment: .leading, spacing: 4) {
                                    HStack {
                                        Text(release.name).fontWeight(.medium)
                                        Spacer()
                                        if let url = release.externalURL {
                                            Link("Publisher’s Site", destination: url)
                                        }
                                    }
                                    Text(release.detail).foregroundStyle(.secondary)
                                        .fixedSize(horizontal: false, vertical: true)
                                }
                            }
                            Text("Publisher results refresh when you check in Updates.")
                                .font(.caption).foregroundStyle(.secondary)
                        }
                        .padding(.top, 7)
                    }
                    .font(.callout)
                }
                Text("Devline uses the toolchains already on this Mac and never changes them. Install and upgrade them with your runtime manager.")
                    .font(.caption).foregroundStyle(.secondary)
            }
            .frame(maxWidth: 580, alignment: .leading)
            .padding(26)
            .frame(maxWidth: .infinity, alignment: .topLeading)
        }
    }

    private func publisherReleases(for runtime: RuntimeToolchain) -> [UpdateItem] {
        let ids = switch runtime.id {
        case "java": ["java", "maven"]
        case "elixir": ["elixir", "erlang"]
        default: [runtime.id]
        }
        return releases.filter { release in ids.contains { release.id == "runtime:\($0)" } }
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
                Button("Install Starter") { prepare(starter) }
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

    private func selectRuntimeIfNeeded() {
        if selection != "docker", !runtimes.contains(where: { $0.id == selection }) {
            selection = runtimes.first(where: { $0.id == projectStack })?.id ?? runtimes.first?.id
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
