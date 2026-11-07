import AppKit
import SwiftUI

struct TemplatesView: View {
    @ObservedObject var model: WorkspaceModel
    @Binding var selection: String?
    @Binding var newStarter: Starter?
    let openDocs: (DocPack) -> Void
    @State private var stack: String?
    @State private var appType: String?
    @State private var buildCandidate: Starter?

    private var starters: [Starter] { model.catalog?.templates ?? [] }
    private var stacks: [String] { Array(Set(starters.map(\.stack))).sorted() }
    private var types: [String] { Array(Set(starters.map(\.appType))).sorted() }
    private var filtered: [Starter] {
        starters.filter { (stack == nil || $0.stack == stack) && (appType == nil || $0.appType == appType) }
    }
    private var families: [StarterFamily] {
        Dictionary(grouping: filtered, by: \.id).values
            .compactMap(StarterFamily.init)
            .sorted { $0.latest.name.localizedStandardCompare($1.latest.name) == .orderedAscending }
    }
    private var selectedStarter: Starter? { filtered.first { $0.reference == selection } }
    private var familySelection: Binding<String?> {
        Binding(
            get: { selectedStarter?.id },
            set: { id in
                if id != selectedStarter?.id {
                    selection = families.first { $0.id == id }?.preferred.reference
                }
            }
        )
    }

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 10) {
                Picker("App Type", selection: $appType) {
                    Text("All Types").tag(nil as String?)
                    ForEach(types, id: \.self) { Text(typeLabel($0)).tag(Optional($0)) }
                }
                .labelsHidden()
                .fixedSize()
                .accessibilityLabel("Filter templates by app type")
                Picker("Stack", selection: $stack) {
                    Text("All Stacks").tag(nil as String?)
                    ForEach(stacks, id: \.self) { Text(displayStack($0)).tag(Optional($0)) }
                }
                .labelsHidden()
                .fixedSize()
                .accessibilityLabel("Filter templates by stack")
                Spacer()
            }
            .padding(.horizontal, 18)
            .padding(.vertical, 13)
            Divider()
            if families.isEmpty {
                ContentUnavailableView(
                    "No Templates",
                    systemImage: "square.on.square",
                    description: Text(starters.isEmpty ? "The catalog has no starter templates yet." : "Choose another app type or stack.")
                )
            } else {
                HSplitView {
                    List(families, selection: familySelection) { family in
                        familyRow(family).tag(family.id)
                    }
                    .listStyle(.inset)
                    .frame(minWidth: 208, idealWidth: 238, maxWidth: 305)
                    Group {
                        if let starter = selectedStarter, let family = families.first(where: { $0.id == starter.id }) {
                            templateDetails(starter, in: family)
                        } else {
                            ContentUnavailableView("Select a Template", systemImage: "square.on.square")
                        }
                    }
                    .frame(minWidth: 350, maxWidth: .infinity, maxHeight: .infinity)
                }
            }
        }
        .onAppear(perform: selectVisibleStarter)
        .onChange(of: filtered.map(\.reference)) { _, _ in selectVisibleStarter() }
        .confirmationDialog("Refresh template dependencies?", isPresented: Binding(
            get: { buildCandidate != nil }, set: { if !$0 { buildCandidate = nil } }
        ), titleVisibility: .visible, presenting: buildCandidate) { starter in
            Button("Build New Release") {
                Task {
                    await model.startJob(CoreRequest("jobs.start", id: starter.reference, kind: "build"))
                }
            }
        } message: { starter in
            Text("Resolve current dependencies for \(starter.name) online, then verify the new release offline before installing it.")
        }
    }

    private func familyRow(_ family: StarterFamily) -> some View {
        HStack(spacing: 10) {
            Image(systemName: family.latest.appType == "desktop" ? "macwindow" : family.latest.appType == "cli" ? "terminal" : "curlybraces")
                .foregroundStyle(.secondary)
                .frame(width: 20)
                .accessibilityHidden(true)
            VStack(alignment: .leading, spacing: 3) {
                Text(family.latest.name).font(.body.weight(.medium))
                Text("\(displayStack(family.latest.stack)) · \(typeLabel(family.latest.appType))")
                    .font(.caption).foregroundStyle(.secondary)
            }
            Spacer(minLength: 0)
            if family.releases.contains(where: \.isInstalled) {
                Image(systemName: "checkmark.circle").foregroundStyle(.secondary)
                    .help("At least one release is installed")
                    .accessibilityLabel("Installed releases")
            }
        }
        .padding(.vertical, 5)
    }

    private func templateDetails(_ starter: Starter, in family: StarterFamily) -> some View {
        let docs = relatedDocs(for: starter)
        return ScrollView {
            VStack(alignment: .leading, spacing: 19) {
                VStack(alignment: .leading, spacing: 7) {
                    Text("\(displayStack(starter.stack)) · \(typeLabel(starter.appType))")
                        .font(.caption.weight(.semibold)).foregroundStyle(.secondary)
                    Text(starter.name).font(.title2.weight(.semibold))
                    Text(starter.description).foregroundStyle(.secondary)
                        .fixedSize(horizontal: false, vertical: true)
                    if let minimumMacos = starter.minimumMacos {
                        Label("macOS \(minimumMacos) or later", systemImage: "desktopcomputer")
                            .font(.caption).foregroundStyle(.secondary)
                    }
                }
                actions(for: starter, docs: docs)
                Divider()
                releaseRow(starter, in: family)
                StarterInventoryView(starter: starter, model: model)
                if starter.verified || !docs.isEmpty {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Included").font(.headline)
                        if starter.verified {
                            Label("Packaged dependencies and an offline test", systemImage: "shippingbox")
                        }
                        if !docs.isEmpty {
                            ForEach(docs, id: \.reference) { pack in
                                HStack {
                                    Label(pack.name, systemImage: "book.closed")
                                    Text(displayDocVersion(pack.version)).font(.caption).foregroundStyle(.secondary)
                                    Spacer()
                                    if pack.installed {
                                        Button("Read Docs") { openDocs(pack) }.controlSize(.small)
                                            .accessibilityLabel("Read \(pack.name) \(displayDocVersion(pack.version))")
                                    } else {
                                        Button("Prepare Docs") { Task { await model.startJob(CoreRequest("jobs.start", id: pack.id, version: pack.version, kind: "docs_prepare")) } }.controlSize(.small)
                                    }
                                }
                            }
                        }
                    }
                    .font(.callout)
                    .foregroundStyle(.secondary)
                }
                DisclosureGroup("Technical Details") {
                    VStack(alignment: .leading, spacing: 6) {
                        Text("Toolchain: \(starter.toolchain) \(starter.minimumVersion)")
                        Text("Target: \(starter.target)")
                        if let minimumMacos = starter.minimumMacos { Text("macOS: \(minimumMacos) or later") }
                    }
                    .font(.callout).foregroundStyle(.secondary)
                    .padding(.top, 6)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .textSelection(.enabled)
                }
                .font(.callout)
            }
            .frame(maxWidth: 580, alignment: .leading)
            .padding(26)
            .frame(maxWidth: .infinity, alignment: .topLeading)
        }
        .id(starter.id)
    }

    private func releaseRow(_ starter: Starter, in family: StarterFamily) -> some View {
        HStack(alignment: .center, spacing: 13) {
            VStack(alignment: .leading, spacing: 4) {
                Text("Release").font(.headline)
                Text(releaseStatus(starter)).font(.callout).foregroundStyle(.secondary)
            }
            Spacer(minLength: 0)
            if family.releases.count > 1 {
                Picker("Template release", selection: $selection) {
                    ForEach(family.releases, id: \.reference) { release in
                        Text(release.version + (release.isInstalled ? " · Installed" : "") + (release.verified ? "" : " · Preview"))
                            .tag(Optional(release.reference))
                    }
                }
                .labelsHidden()
                .fixedSize()
                .accessibilityLabel("Template release")
            } else {
                Text(starter.version).font(.callout.monospacedDigit())
            }
        }
    }

    private func actions(for starter: Starter, docs: [DocPack]) -> some View {
        HStack(spacing: 9) {
            if starter.verified, starter.isInstalled {
                Button("New Project…") { newStarter = starter }
                    .buttonStyle(.borderedProminent)
            } else if starter.verified {
                Button("Prepare Starter") { install(starter, withDocs: !starter.docs.isEmpty) }
                    .buttonStyle(.borderedProminent)
                    .help("Install the starter and its related documentation from the local catalog")
            } else {
                Text("Project creation requires an offline-verified release.")
                    .font(.callout).foregroundStyle(.secondary)
            }
            if starter.isInstalled && docs.contains(where: { !$0.installed }) {
                Button("Install Related Docs") { install(starter, withDocs: true) }
            }
            Menu {
                if !starter.isInstalled && !starter.docs.isEmpty {
                    Button("Install Template Only") { install(starter, withDocs: false) }
                }
                Button("Refresh Dependencies…") { buildCandidate = starter }
                if starter.isInstalled {
                    Button("Verify Offline Now") { Task { await model.startJob(CoreRequest("jobs.start", id: starter.reference, kind: "verify")) } }
                }
            } label: {
                Label("More Actions", systemImage: "ellipsis.circle")
                    .labelStyle(.iconOnly)
            }
            .menuStyle(.borderlessButton)
            .fixedSize()
            .accessibilityLabel("More Template Actions")
        }
        .disabled(model.operation != nil)
    }

    private func relatedDocs(for starter: Starter) -> [DocPack] {
        let packs = model.catalog?.docs ?? []
        return starter.docs.compactMap { reference in
            return packs.first { $0.reference == reference }
                ?? packs.filter { $0.id == reference }.max {
                    $0.version.compare($1.version, options: .numeric) == .orderedAscending
                }
        }
    }

    private func releaseStatus(_ starter: Starter) -> String {
        if !starter.verified { return starter.isInstalled ? "Installed preview" : "Preview" }
        return starter.isInstalled ? "Installed · Packaged dependencies" : "Available · Packaged dependencies"
    }

    private func selectVisibleStarter() {
        if selectedStarter == nil {
            selection = families.first?.preferred.reference
        }
    }

    private func install(_ starter: Starter, withDocs: Bool) {
        Task {
            if withDocs {
                await model.startJob(CoreRequest("jobs.start", id: starter.reference, kind: "prepare"))
            } else {
                await model.mutate([CoreRequest("templates.install", id: starter.reference)],
                                   label: "Installing \(starter.name)", completion: "Installed \(starter.name) \(starter.version).")
            }
        }
    }

    private struct StarterFamily: Identifiable {
        let id: String
        let latest: Starter
        let releases: [Starter]

        var preferred: Starter { releases.first(where: \.verified) ?? latest }

        init?(_ starters: [Starter]) {
            let ordered = starters.sorted { $0.version.compare($1.version, options: .numeric) == .orderedDescending }
            guard let latest = ordered.first else { return nil }
            self.id = latest.id
            self.latest = latest
            self.releases = ordered
        }
    }
}

struct NewProjectView: View {
    @Environment(\.dismiss) private var dismiss
    @ObservedObject var model: WorkspaceModel
    let starter: Starter
    @ObservedObject var infrastructure: InfrastructureLibrary
    let openDocs: (String) -> Void
    @State private var name = ""
    @State private var parentFolder = FileManager.default.homeDirectoryForCurrentUser.appendingPathComponent("Documents")
    @State private var readiness: Readiness?
    @State private var readinessError: String?
    @State private var checkRevision = 0
    @State private var creating = false
    @State private var creationJob: ProjectCreationJob?
    @State private var cancelling = false
    @State private var createError: String?
    @State private var project: Project?
    @State private var copied = false
    @State private var services: Set<String> = []
    @State private var clients: Set<String> = []
    @State private var ports: [String: Int] = [:]
    @State private var showServices = false
    @FocusState private var nameFocused: Bool

    private var projectName: String { name.trimmingCharacters(in: .whitespacesAndNewlines) }
    private var validName: Bool {
        projectName.utf8.count <= 64 &&
            projectName.range(of: "^[a-z](?:[a-z0-9-]*[a-z0-9])?$", options: .regularExpression) != nil
    }
    private var destination: URL { parentFolder.appendingPathComponent(projectName, isDirectory: true) }
    private var portError: String? {
        let edited = (infrastructure.catalog?.services ?? []).filter { services.contains($0.reference) }
            .flatMap { service in service.ports.compactMap { ports["\(service.id):\($0.id)"] } }
        if edited.contains(where: { !(1...65535).contains($0) }) { return "Use a host port between 1 and 65535." }
        if Set(edited).count != edited.count { return "Each edited host port must be different." }
        return nil
    }

    var body: some View {
        VStack(spacing: 0) {
            HStack(alignment: .top, spacing: 12) {
                Image(systemName: project == nil ? "folder.badge.plus" : "checkmark.circle.fill")
                    .font(.system(size: 27)).foregroundStyle(project == nil ? Color.accentColor : .green)
                    .accessibilityHidden(true)
                VStack(alignment: .leading, spacing: 4) {
                    Text(project == nil ? "New Project" : "Project Created").font(.title2.weight(.semibold))
                    Text("\(starter.name) · Release \(starter.version)")
                        .font(.callout).foregroundStyle(.secondary)
                }
                Spacer()
            }
            .padding(21)
            Divider()
            if let project { completion(project) }
            else { configuration }
            Divider()
            footer
        }
        .frame(width: 640)
        .interactiveDismissDisabled(creating)
        .task(id: checkRevision) { await runDoctor() }
        .onAppear { nameFocused = true; services = Set(starter.services ?? []); showServices = !services.isEmpty }
        .task { await infrastructure.refresh() }
        .task(id: creationJob?.id) { await followCreation() }
    }

    private var configuration: some View {
        VStack(alignment: .leading, spacing: 15) {
            Grid(alignment: .leading, horizontalSpacing: 13, verticalSpacing: 10) {
                GridRow {
                    Text("Name").gridColumnAlignment(.trailing)
                    TextField("orders-api", text: $name)
                        .textFieldStyle(.roundedBorder)
                        .focused($nameFocused)
                        .accessibilityLabel("Project name")
                }
                GridRow {
                    Text("Folder")
                    HStack {
                        Text((parentFolder.path(percentEncoded: false) as NSString).abbreviatingWithTildeInPath)
                            .lineLimit(1).truncationMode(.middle).foregroundStyle(.secondary)
                            .help(parentFolder.path(percentEncoded: false))
                        Spacer()
                        Button("Choose…", action: chooseFolder)
                    }
                }
            }
            if !name.isEmpty && !validName {
                Text("Use up to 64 lowercase letters, digits, and hyphens. Start with a letter and end with a letter or digit.")
                    .font(.caption).foregroundStyle(.secondary)
            }
            readinessPanel
            DisclosureGroup(isExpanded: $showServices) {
                ServiceSelectionView(library: infrastructure, selected: $services, ports: $ports, clients: $clients,
                    required: Set(starter.services ?? []), runtime: starter.stack, starterReference: starter.reference)
                    .frame(height: 230).padding(.top, 8)
            } label: {
                Text(services.isEmpty ? "Add Local Services" : "Local Services · \(services.count) Selected").font(.callout.weight(.medium))
            }
            if !services.isEmpty {
                Text("Devline saves these image archives while connected so your services can start offline. You can add more services from the project screen later.")
                    .font(.caption).foregroundStyle(.secondary)
            }
            if let issue = createError ?? portError {
                Label(issue, systemImage: "exclamationmark.circle")
                    .font(.callout).foregroundStyle(.red)
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
        .padding(20)
        .disabled(creating)
    }

    private var readinessPanel: some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 9) {
                if let readiness {
                    HStack(spacing: 8) {
                        Image(systemName: readiness.ready ? "checkmark.circle.fill" : "exclamationmark.circle.fill")
                            .foregroundStyle(readiness.ready ? Color.green : .orange)
                            .accessibilityHidden(true)
                        Text(readiness.readyOffline == true ? "Starter Verified Offline" : readiness.ready ? "Ready to Create · Verification Pending" : "Setup Required")
                            .font(.callout.weight(.semibold))
                    }
                    let failures = readiness.checks.filter { !$0.ok }
                    ForEach(failures.indices, id: \.self) { index in
                        let check = failures[index]
                        VStack(alignment: .leading, spacing: 3) {
                            Text(check.name).font(.callout.weight(.medium))
                            Text(check.detail).font(.caption).foregroundStyle(.secondary)
                                .fixedSize(horizontal: false, vertical: true)
                        }
                    }
                    let passed = readiness.checks.filter(\.ok)
                    if !passed.isEmpty {
                        DisclosureGroup("Readiness Details") {
                            VStack(alignment: .leading, spacing: 7) {
                                ForEach(passed.indices, id: \.self) { index in
                                    let check = passed[index]
                                    Text("\(check.name): \(check.detail)")
                                        .fixedSize(horizontal: false, vertical: true)
                                }
                            }
                            .font(.caption).foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .padding(.top, 7)
                            .textSelection(.enabled)
                        }
                        .font(.caption)
                    }
                } else if let readinessError {
                    Label(readinessError, systemImage: "exclamationmark.circle")
                        .font(.callout).foregroundStyle(.red)
                } else {
                    HStack(spacing: 8) {
                        ProgressView().controlSize(.small)
                        Text("Checking toolchain and template…").font(.callout)
                    }
                }
                if readiness?.ready == false || readinessError != nil {
                    Button("Check Again") { checkRevision += 1 }.controlSize(.small)
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(4)
        }
    }

    private func completion(_ project: Project) -> some View {
        VStack(alignment: .leading, spacing: 13) {
            Text(project.path).textSelection(.enabled)
                .font(.callout.monospaced())
                .fixedSize(horizontal: false, vertical: true)
            Text("Open your project to edit the code, run the app offline, and manage its local services.")
                .font(.callout).foregroundStyle(.secondary)
            if !project.testCommand.isEmpty {
                Divider()
                Text("Offline tests passed").font(.headline)
                Text(project.testCommand)
                    .font(.system(.callout, design: .monospaced))
                    .textSelection(.enabled)
                Button(copied ? "Copied" : "Copy Test Command") {
                    copy("cd \(shellQuote(project.path)) && \(project.testCommand)")
                }
            }
            if !services.isEmpty {
                Label("Service images are archived and pinned. Start them from the project workspace.", systemImage: "server.rack")
                    .font(.callout).foregroundStyle(.secondary)
            }
            if let createError { Text(createError).font(.caption).foregroundStyle(.orange).textSelection(.enabled) }
        }
        .padding(21)
        .frame(maxWidth: .infinity, alignment: .leading)
    }

    private var footer: some View {
        HStack(spacing: 10) {
            if let project {
                Button("Show in Finder") {
                    NSWorkspace.shared.activateFileViewerSelecting([URL(fileURLWithPath: project.path)])
                }
                Spacer()
                Button("Open Project") { openDocs(project.path) }.keyboardShortcut(.defaultAction).buttonStyle(.borderedProminent)
            } else {
                if validName {
                    Button(copied ? "Copied" : "Copy CLI Command") {
                        let serviceFlags = services.sorted().map { " --service \(shellQuote($0))" }.joined()
                        let clientFlags = clients.sorted().map { " --client \(shellQuote($0))" }.joined()
                        let portFlags = ports.sorted { $0.key < $1.key }.map { " --port \(shellQuote("\($0.key)=\($0.value)"))" }.joined()
                        copy("devline new \(shellQuote(starter.reference)) \(shellQuote(destination.path(percentEncoded: false)))\(serviceFlags)\(clientFlags)\(portFlags)")
                    }
                    .disabled(creating)
                }
                Spacer()
                if creating {
                    ProgressView().controlSize(.small)
                    Text(cancelling ? "Cancelling…" : creationJob?.stage ?? "Starting creation…")
                        .font(.caption).foregroundStyle(.secondary).lineLimit(2)
                }
                Button(creating ? "Cancel Creation" : "Cancel") {
                    if creating { Task { await cancelCreation() } } else { dismiss() }
                }
                    .keyboardShortcut(.cancelAction)
                    .disabled(creating && (creationJob == nil || cancelling))
                Button("Create Project") { Task { await create() } }
                    .keyboardShortcut(.defaultAction)
                    .buttonStyle(.borderedProminent)
                    .disabled(!validName || readiness?.ready != true || creating || portError != nil)
            }
        }
        .padding(16)
    }

    private func chooseFolder() {
        let panel = NSOpenPanel()
        panel.title = "Choose Project Folder"
        panel.prompt = "Choose"
        panel.directoryURL = parentFolder
        panel.canChooseDirectories = true
        panel.canChooseFiles = false
        panel.canCreateDirectories = true
        panel.allowsMultipleSelection = false
        if panel.runModal() == .OK, let folder = panel.url { parentFolder = folder }
    }

    private func copy(_ value: String) {
        NSPasteboard.general.clearContents()
        if NSPasteboard.general.setString(value, forType: .string) { copied = true }
    }

    private func runDoctor() async {
        readiness = nil
        readinessError = nil
        do {
            readiness = try await Core.request(CoreRequest("doctor", id: starter.reference))
        } catch is CancellationError {
        } catch {
            readinessError = error.localizedDescription
        }
    }

    private func create() async {
        guard validName, readiness?.ready == true, !creating, portError == nil else { return }
        creating = true
        cancelling = false
        createError = nil
        copied = false
        do {
            creationJob = try await Core.request(CoreRequest("jobs.start", id: starter.reference,
                destination: destination.path(percentEncoded: false), kind: "new_project",
                services: services.sorted(), clients: clients.sorted(), ports: ports))
            await model.refreshJobs()
        } catch {
            creating = false
            createError = error.localizedDescription
        }
    }

    private func followCreation() async {
        guard let id = creationJob?.id else { return }
        var offset = 0
        while !Task.isCancelled {
            do {
                let job: ProjectCreationJob = try await Core.request(CoreRequest("jobs.status", offset: offset, jobId: id))
                guard !Task.isCancelled, creationJob?.id == id else { return }
                creationJob = job
                offset = job.nextOffset ?? offset
                if !job.isActive {
                    creating = false
                    cancelling = false
                    if job.state == "succeeded", let result = job.result {
                        project = result
                    } else {
                        createError = job.error ?? (job.state == "cancelled" ? "Creation cancelled. Prepared library items are kept." : "Creation did not finish. Review its log in Activity.")
                    }
                    return
                }
                try await Task.sleep(for: .milliseconds(500))
            } catch is CancellationError { return }
            catch {
                createError = "Couldn’t read creation progress: \(error.localizedDescription). The job remains available in Activity."
                do { try await Task.sleep(for: .seconds(2)) } catch { return }
            }
        }
    }

    private func cancelCreation() async {
        guard let id = creationJob?.id, !cancelling else { return }
        cancelling = true
        do { let _: Acknowledgment = try await Core.request(CoreRequest("jobs.cancel", jobId: id)) }
        catch { cancelling = false; createError = error.localizedDescription }
    }
}

private struct ProjectCreationJob: Decodable {
    let id, state, stage: String
    let error: String?
    let nextOffset: Int?
    let result: Project?
    var isActive: Bool { state == "queued" || state == "running" }
}

private func typeLabel(_ type: String) -> String {
    switch type.lowercased() {
    case "api", "http-api": "HTTP API"
    case "cli": "CLI"
    default: type.replacingOccurrences(of: "-", with: " ").capitalized
    }
}
