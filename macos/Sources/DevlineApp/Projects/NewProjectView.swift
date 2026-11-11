import AppKit
import SwiftUI

struct NewProjectView: View {
    @Environment(\.dismiss) private var dismiss
    @ObservedObject var model: WorkspaceModel
    let starter: Starter
    @ObservedObject var infrastructure: InfrastructureLibrary
    let open: (String) -> Void
    @State private var name = ""
    @State private var parentFolder = NewProjectView.rememberedFolder()
    @State private var readiness: Readiness?
    @State private var readinessError: String?
    @State private var checkRevision = 0
    @State private var creating = false
    @State private var creationJob: ProjectCreationJob?
    @State private var cancelling = false
    @State private var createError: String?
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
    private var nameTaken: Bool {
        validName && FileManager.default.fileExists(atPath: destination.path(percentEncoded: false))
    }
    /// Creating a project installs its starter first, so a missing install is not a blocker.
    private var blockingChecks: [Readiness.Check] {
        readiness?.checks.filter { !$0.ok && $0.name != "Installed release" } ?? []
    }
    private var canCreate: Bool { readiness != nil && blockingChecks.isEmpty }
    private var portError: String? {
        let selected = (infrastructure.catalog?.services ?? []).filter { services.contains($0.reference) }
        return servicePortError(ports, for: selected)
    }

    var body: some View {
        VStack(spacing: 0) {
            HStack(alignment: .top, spacing: 12) {
                Image(systemName: appTypeSymbol(starter.appType))
                    .font(.system(size: 17, weight: .medium)).foregroundStyle(stackTint(starter.stack))
                    .frame(width: 38, height: 38)
                    .background(stackTint(starter.stack).opacity(0.15), in: RoundedRectangle(cornerRadius: 9))
                    .accessibilityHidden(true)
                VStack(alignment: .leading, spacing: 3) {
                    Text("New \(starter.name)").font(.title2.weight(.semibold))
                    Text("\(displayStack(starter.stack)) · \(displayAppType(starter.appType)) · \(model.catalog?.summary(of: starter) ?? starter.description)")
                        .font(.callout).foregroundStyle(.secondary).lineLimit(2)
                }
                Spacer()
            }
            .padding(21)
            Divider()
            configuration
            Divider()
            footer
        }
        .frame(width: 640)
        .interactiveDismissDisabled(creating)
        .task(id: checkRevision) { await runDoctor() }
        .onAppear {
            if name.isEmpty { name = suggestedName() }
            nameFocused = true
            services = Set(starter.services ?? [])
            showServices = !services.isEmpty
        }
        .task { await infrastructure.loadIfNeeded(after: model.completedInfrastructureJobIDs) }
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
            } else if nameTaken {
                Text("A folder with this name already exists here. Choose another name or folder.")
                    .font(.caption).foregroundStyle(.orange)
            }
            readinessPanel
            DisclosureGroup(isExpanded: $showServices) {
                VStack(alignment: .leading, spacing: 8) {
                    ServiceSelectionView(library: infrastructure, selected: $services, ports: $ports, clients: $clients,
                        required: Set(starter.services ?? []), runtime: starter.stack, starterReference: starter.reference)
                        .frame(height: 230)
                    Text("Devline saves selected image archives while connected so services can start offline. You can add more services from the project screen later.")
                        .font(.caption).foregroundStyle(.secondary)
                }.padding(.top, 8)
            } label: {
                Text(services.isEmpty ? "Add Local Services" : "Local Services · \(services.count) Selected").font(.callout.weight(.medium))
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
                        Image(systemName: canCreate ? "checkmark.circle.fill" : "exclamationmark.circle.fill")
                            .foregroundStyle(canCreate ? Color.green : .orange)
                            .accessibilityHidden(true)
                        VStack(alignment: .leading, spacing: 2) {
                            Text(canCreate ? "This Mac is ready for it" : "This Mac needs something first")
                                .font(.callout.weight(.semibold))
                            if canCreate {
                                Text(readiness.readyOffline == true
                                     ? "This starter already passed its tests here with the network blocked."
                                     : "Devline tests the new project with the network blocked before opening it.")
                                    .font(.caption).foregroundStyle(.secondary)
                            }
                        }
                    }
                    let failures = blockingChecks
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
                        DisclosureGroup("What Devline checked") {
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
                        Text("Checking this Mac…").font(.callout)
                    }
                }
                if (readiness != nil && !canCreate) || readinessError != nil {
                    Button("Check Again") { checkRevision += 1 }.controlSize(.small)
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(4)
        }
    }

    private var footer: some View {
        HStack(spacing: 10) {
            if creating {
                ProgressView().controlSize(.small)
                Text(cancelling ? "Cancelling…" : creationJob?.stage ?? "Starting…")
                    .font(.callout).foregroundStyle(.secondary).lineLimit(2)
            } else if validName {
                Button(copied ? "Copied" : "Copy CLI Command") {
                    let serviceFlags = services.sorted().map { " --service \(shellQuote($0))" }.joined()
                    let clientFlags = clients.sorted().map { " --client \(shellQuote($0))" }.joined()
                    let portFlags = ports.sorted { $0.key < $1.key }.map { " --port \(shellQuote("\($0.key)=\($0.value)"))" }.joined()
                    copy("devline new \(shellQuote(starter.reference)) \(shellQuote(destination.path(percentEncoded: false)))\(serviceFlags)\(clientFlags)\(portFlags)")
                }
                .buttonStyle(.link).font(.callout)
            }
            Spacer()
            Button(creating ? "Stop" : "Cancel") {
                if creating { Task { await cancelCreation() } } else { dismiss() }
            }
                .keyboardShortcut(.cancelAction)
                .disabled(creating && (creationJob == nil || cancelling))
            Button("Create and Open") { Task { await create() } }
                .keyboardShortcut(.defaultAction)
                .buttonStyle(.borderedProminent)
                .disabled(!validName || nameTaken || !canCreate || creating || portError != nil)
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
        if panel.runModal() == .OK, let folder = panel.url {
            parentFolder = folder
            UserDefaults.standard.set(folder.path(percentEncoded: false), forKey: Self.folderKey)
            if nameTaken { name = suggestedName() }
        }
    }

    private static let folderKey = "devline.projectsFolder"

    /// The folder used last time, or the first conventional one that exists.
    private static func rememberedFolder() -> URL {
        let files = FileManager.default
        let home = files.homeDirectoryForCurrentUser
        var candidates = ["Developer", "Projects", "Code", "Documents"].map { home.appendingPathComponent($0, isDirectory: true) }
        if let saved = UserDefaults.standard.string(forKey: folderKey) {
            candidates.insert(URL(fileURLWithPath: saved, isDirectory: true), at: 0)
        }
        var isDirectory: ObjCBool = false
        return candidates.first {
            files.fileExists(atPath: $0.path(percentEncoded: false), isDirectory: &isDirectory) && isDirectory.boolValue
        } ?? home
    }

    /// The starter's own name, numbered when that folder already exists.
    private func suggestedName() -> String {
        let base = projectSlug(for: starter.id)
        let taken = { (candidate: String) in
            FileManager.default.fileExists(atPath: parentFolder.appendingPathComponent(candidate).path(percentEncoded: false))
        }
        guard taken(base) else { return base }
        return (2...99).lazy.map { "\(base)-\($0)" }.first { !taken($0) } ?? base
    }

    private func copy(_ value: String) {
        if copyToPasteboard(value) { copied = true }
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
        guard validName, !nameTaken, canCreate, !creating, portError == nil else { return }
        creating = true
        cancelling = false
        createError = nil
        copied = false
        do {
            let job: ProjectCreationJob = try await Core.request(CoreRequest("jobs.start", id: starter.reference,
                destination: destination.path(percentEncoded: false), kind: "new_project",
                services: services.sorted(), clients: clients.sorted(), ports: ports))
            creationJob = job
            await model.refreshJobs(startedJobID: job.id)
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
                        open(result.path)
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
