import SwiftUI

struct ConfigureServicesView: View {
    @Environment(\.dismiss) private var dismiss
    @ObservedObject var model: WorkspaceModel
    @ObservedObject var library: InfrastructureLibrary
    let path: String
    let runtime: String?
    let existing: InfrastructureStack?
    let suggested: [String]
    let completion: () -> Void
    @State private var selected: Set<String> = []
    @State private var clients: Set<String> = []
    @State private var ports: [String: Int] = [:]
    @State private var saving = false
    @State private var error: String?
    private var existingIDs: Set<String> { Set(existing?.services.map(\.id) ?? []) }
    private var adding: Bool { existing?.configured == true }
    private var selectedServices: [InfrastructureService] {
        (library.catalog?.services ?? []).filter { selected.contains($0.reference) && !existingIDs.contains($0.id) }
    }
    private var portError: String? {
        servicePortError(ports, for: selectedServices, occupied: existing?.services.flatMap { $0.ports.map(\.hostPort) } ?? [])
    }
    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text(adding ? "Add Local Services" : "Choose Local Services").font(.title2.weight(.semibold))
            Text(adding ? "Keep your existing services, ports, credentials, and data. Devline prepares the new image archives after adding them." : "Choose what your app needs. Devline saves exact versions and prepares their image archives so they can start offline.")
                .font(.callout).foregroundStyle(.secondary)
            if let existing, !existing.services.isEmpty {
                Text("Already configured: " + existing.services.map { service in
                    let name = library.catalog?.services.first { $0.id == service.id }?.name ?? service.id
                    return "\(name) \(service.version)"
                }.joined(separator: ", ")).font(.caption).foregroundStyle(.secondary)
            }
            ServiceSelectionView(library: library, selected: $selected, ports: $ports, clients: $clients,
                required: Set(existing?.services.map(\.reference) ?? []), runtime: runtime, projectPath: path,
                configuredServices: existing?.services ?? []).frame(height: 340)
            if let error = error ?? portError { Text(error).font(.callout).foregroundStyle(.red).textSelection(.enabled) }
            HStack {
                Text("Local connections only · 127.0.0.1").font(.caption).foregroundStyle(.secondary)
                Spacer()
                if saving { ProgressView().controlSize(.small) }
                Button("Cancel") { dismiss() }.keyboardShortcut(.cancelAction).disabled(saving)
                Button(adding ? "Add & Prepare" : "Configure & Prepare") { Task { await save() } }
                    .buttonStyle(.borderedProminent).keyboardShortcut(.defaultAction)
                    .disabled((selected.subtracting(Set(existing?.services.map(\.reference) ?? [])).isEmpty && clients.isEmpty) || saving || portError != nil)
            }
        }.padding(24).frame(width: 660).interactiveDismissDisabled(saving)
            .task {
                selected = Set(existing?.services.map(\.reference) ?? [])
                selected.formUnion(suggested.filter { reference in !existingIDs.contains(String(reference.split(separator: "@", maxSplits: 1)[0])) })
                await library.refresh()
            }
    }
    private func save() async {
        saving = true; error = nil
        defer { saving = false }
        if await model.startJob(CoreRequest("jobs.start", path: path, kind: "infra_stack_add", services: selected.sorted(), clients: clients.sorted(), ports: ports), inspect: false) {
            completion()
        } else { error = model.error }
    }
}

func servicePortError(_ overrides: [String: Int], for services: [InfrastructureService], occupied: [Int] = []) -> String? {
    let edited = services.flatMap { service in service.ports.compactMap { overrides["\(service.id):\($0.id)"] } }
    if edited.contains(where: { !(1...65535).contains($0) }) { return "Use a host port between 1 and 65535." }
    let allPorts = edited + occupied
    if Set(allPorts).count != allPorts.count { return "An edited host port conflicts with another edited or configured port. Choose a different port." }
    return nil
}

struct ServiceSelectionView: View {
    @ObservedObject var library: InfrastructureLibrary
    @Binding var selected: Set<String>
    @Binding var ports: [String: Int]
    @Binding var clients: Set<String>
    var required: Set<String> = []
    var runtime: String?
    var projectPath: String?
    var starterReference: String?
    var configuredServices: [InfrastructureStack.Service] = []
    @State private var query = ""
    @State private var clientCatalog: ServiceClientCatalog?
    @State private var clientError: String?
    @State private var loadingClients = false
    private var clientContext: String { [runtime, projectPath, starterReference].compactMap { $0 }.joined(separator: "|") }
    private var services: [InfrastructureService] {
        (library.catalog?.services ?? []).filter { service in
            !configuredServices.contains(where: { $0.id == service.id && $0.reference != service.reference }) &&
            (query.isEmpty || "\(service.name) \(service.category) \(service.description)".localizedCaseInsensitiveContains(query))
        }
    }
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Service images run locally. Select optional client libraries below when you want to add them to your code.")
                .font(.caption).foregroundStyle(.secondary)
            Text("Shown ports are preferred. Devline chooses a free port when needed unless you edit it.")
                .font(.caption).foregroundStyle(.secondary)
            TextField("Find a service", text: $query).textFieldStyle(.roundedBorder)
                .accessibilityLabel("Find a local service")
            ControlList(horizontalPadding: 8) {
                ForEach(services, id: \.reference) { service in
                    ControlListRow { serviceRow(service) }
                }
            }
            .overlay {
                if library.loading && library.catalog == nil { ProgressView() }
                else if services.isEmpty { Text("No matching services").font(.callout).foregroundStyle(.secondary) }
            }
            if let clientError { Text(clientError).font(.caption).foregroundStyle(.orange) }
        }
        .task(id: clientContext) { await loadClients() }
    }

    private func serviceRow(_ service: InfrastructureService) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Toggle(isOn: Binding(get: { selected.contains(service.reference) }, set: { select(service, enabled: $0) })) {
                HStack {
                    Label(service.name, systemImage: service.symbol)
                    Spacer()
                    Text(service.version).font(.caption).foregroundStyle(.secondary)
                    if required.contains(service.reference) {
                        Text(configuredServices.contains { $0.reference == service.reference } ? "Configured" : "Required").font(.caption).foregroundStyle(.secondary)
                    }
                    Text(imageStatus(service)).font(.caption).foregroundStyle(.secondary)
                }
            }.toggleStyle(.checkbox)
                .disabled(required.contains(service.reference) || required.contains { $0.hasPrefix(service.id + "@") && $0 != service.reference })
            Text(service.description).font(.caption).foregroundStyle(.secondary).lineLimit(2).padding(.leading, 22)
            if selected.contains(service.reference) {
                clientOptions(service)
                ForEach(service.ports) { port in
                    HStack {
                        Text("\(port.name) port").font(.caption).foregroundStyle(.secondary)
                        Spacer()
                        TextField("Port", value: Binding(get: { ports["\(service.id):\(port.id)"] ?? configuredServices.first { $0.id == service.id }?.ports.first { $0.id == port.id }?.hostPort ?? port.hostPort }, set: { ports["\(service.id):\(port.id)"] = $0 }), format: .number.grouping(.never))
                            .textFieldStyle(.roundedBorder).frame(width: 90).accessibilityLabel("\(service.name) \(port.name) host port")
                            .disabled(configuredServices.contains { $0.id == service.id })
                    }.padding(.leading, 22)
                }
            }
        }
    }

    @ViewBuilder private func clientOptions(_ service: InfrastructureService) -> some View {
        VStack(alignment: .leading, spacing: 7) {
            if loadingClients {
                Text("Checking optional client libraries…").font(.caption).foregroundStyle(.secondary)
            } else if let entry = clientCatalog?.services.first(where: { $0.id == service.id }) {
                ForEach(entry.libraries) { client in
                    VStack(alignment: .leading, spacing: 4) {
                        if client.builtin {
                            Label("\(client.name) · Built into the runtime", systemImage: "checkmark.circle")
                                .font(.caption).foregroundStyle(.secondary)
                        } else if client.installed {
                            Label("\(client.name) · \(starterReference == nil ? "Already in this project" : "Included in starter")", systemImage: "checkmark.circle")
                                .font(.caption).foregroundStyle(.secondary)
                        } else if client.supported {
                            Toggle("Add \(client.name) to project", isOn: Binding(get: { clients.contains(client.id) }, set: {
                                if $0 { clients.insert(client.id) } else { clients.remove(client.id) }
                            })).toggleStyle(.checkbox).font(.callout)
                            Text(client.offlineAvailable ? "Packages available offline" : "A connection is needed to prepare these packages")
                                .font(.caption).foregroundStyle(.secondary)
                        } else {
                            Text(client.name).font(.caption.weight(.medium))
                        }
                        if !client.installed && !client.packageSummary.isEmpty { Text(client.packageSummary).font(.caption.monospaced()).foregroundStyle(.secondary) }
                        if !client.installed || client.builtin {
                            Text(client.detail).font(.caption).foregroundStyle(.secondary).fixedSize(horizontal: false, vertical: true)
                        }
                    }
                }
            }
        }.padding(.leading, 22)
    }

    private func loadClients() async {
        guard let runtime else { return }
        loadingClients = true
        clientError = nil
        defer { loadingClients = false }
        do {
            let next: ServiceClientCatalog = try await Core.request(CoreRequest("infra.clients", id: starterReference, stack: runtime, path: projectPath))
            guard !Task.isCancelled else { return }
            clientCatalog = next
        } catch is CancellationError {} catch { self.clientError = "Client suggestions are unavailable: \(error.localizedDescription)" }
    }
    private func select(_ service: InfrastructureService, enabled: Bool) {
        let portPrefix = service.id + ":"
        if enabled {
            selected = selected.filter { !$0.hasPrefix(service.id + "@") }
            selected.insert(service.reference)
            let availablePorts = Set(service.ports.map { portPrefix + $0.id })
            ports = ports.filter { !$0.key.hasPrefix(portPrefix) || availablePorts.contains($0.key) }
        } else {
            selected.remove(service.reference)
            clients.subtract(clientCatalog?.services.first { $0.id == service.id }?.libraries.map(\.id) ?? [])
            ports = ports.filter { !$0.key.hasPrefix(portPrefix) }
        }
    }
    private func imageStatus(_ service: InfrastructureService) -> String {
        if let image = library.image(service.reference), image.archived {
            return image.verifiedArchive ? "Archived" : "Verify archive"
        }
        return service.image.compressedBytes.map(storageSize) ?? "Download needed"
    }
}
