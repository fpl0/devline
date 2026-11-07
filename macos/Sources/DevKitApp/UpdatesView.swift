import AppKit
import SwiftUI

struct UpdateSnapshot: Decodable, Sendable {
    let checkedAt: TimeInterval?
    let items: [UpdateItem]
}

struct UpdateItem: Decodable, Identifiable, Sendable {
    enum Kind: String, CaseIterable, Decodable, Identifiable, Sendable {
        case docs, template, runtime, app

        var id: String { rawValue }

        var title: String {
            switch self {
            case .docs: "Documentation"
            case .template: "Templates"
            case .runtime: "Runtimes"
            case .app: "Devline App"
            }
        }
    }

    enum State: String, Decodable, Sendable {
        case current, available, unchecked, error, unconfigured

        var label: String {
            switch self {
            case .current: "Up to Date"
            case .available: "Update Available"
            case .unchecked: "Unchecked"
            case .error: "Check Failed"
            case .unconfigured: "Checks Disabled"
            }
        }

        var icon: String {
            switch self {
            case .current: "checkmark.circle"
            case .available: "arrow.down.circle.fill"
            case .unchecked: "clock"
            case .error: "exclamationmark.circle"
            case .unconfigured: "shippingbox"
            }
        }

        var sortOrder: Int {
            switch self {
            case .available: 0
            case .error: 1
            case .unchecked: 2
            case .current, .unconfigured: 3
            }
        }
    }

    let id: String
    let kind: Kind
    let name: String
    let currentVersion: String?
    let availableVersion: String?
    let state: State
    let detail: String
    let action: String?
    let reference: String?
    let url: String?

    var versionSummary: String? {
        if let currentVersion, let availableVersion, currentVersion != availableVersion {
            return "\(currentVersion) → \(availableVersion)"
        }
        if let currentVersion { return currentVersion }
        if let availableVersion { return "Available \(availableVersion)" }
        return nil
    }

    var docInstallRequest: CoreRequest? {
        guard action == "install_docs", let reference else { return nil }
        let parts = reference.split(separator: "@", maxSplits: 1, omittingEmptySubsequences: false)
        guard let id = parts.first, !id.isEmpty else { return nil }
        let version = parts.count == 2 ? String(parts[1]) : availableVersion
        guard let version, !version.isEmpty else { return nil }
        return CoreRequest("docs.install", id: String(id), version: version)
    }

    var externalURL: URL? {
        guard action == "open_url", let url else { return nil }
        return publicHTTPSURL(url)
    }
}

struct UpdatesView: View {
    private enum Filter: Hashable {
        case allItems, available
    }

    @ObservedObject var model: WorkspaceModel
    var checkOnOpen = false
    @State private var snapshot: UpdateSnapshot?
    @State private var filter: Filter = .allItems
    @State private var checking = false
    @State private var loading = false
    @State private var loadError: String?
    @State private var rebuildTarget: UpdateItem?
    @State private var configuringChannel = false

    var body: some View {
        TabView {
            DocDistributionView(model: model, checkOnOpen: checkOnOpen).tabItem { Text("Documentation") }
            sourceUpdates.tabItem { Text("Starters and Runtimes") }
            ImageUpdatesView(model: model).tabItem { Text("Infrastructure") }
            Form { AppUpdatesSettings() }.formStyle(.grouped).tabItem { Text("Devline App") }
        }.padding(.top, 8)
    }

    private var sourceUpdates: some View {
        VStack(spacing: 0) {
            header
            if !documentationUpdates.isEmpty {
                HStack(spacing: 12) {
                    Label("\(documentationUpdates.count) documentation updates ready offline", systemImage: "internaldrive")
                        .font(.callout).foregroundStyle(.secondary)
                    Spacer()
                    Button("Install Documentation Updates") {
                        let requests = documentationUpdates
                        Task {
                            _ = await model.mutate(requests, label: "Updating documentation",
                                                   completion: "Installed \(requests.count) documentation updates. Previous versions remain available.")
                            await loadStatus()
                        }
                    }.disabled(checking || loading || model.operation != nil)
                }.padding(.horizontal, 18).padding(.bottom, 14)
            }
            Divider()
            if let snapshot {
                if let loadError {
                    Label(loadError, systemImage: "exclamationmark.circle")
                        .font(.callout).foregroundStyle(.secondary)
                        .padding(.horizontal, 18).padding(.vertical, 9)
                        .frame(maxWidth: .infinity, alignment: .leading)
                    Divider()
                }
                if filter == .available {
                    let available = snapshot.items.filter { $0.state == .available && ($0.kind == .template || $0.kind == .runtime) }
                    if available.isEmpty {
                        availableEmptyState(snapshot)
                    } else {
                        updateList(available)
                    }
                } else {
                    updateList(snapshot.items.filter { $0.kind == .template || $0.kind == .runtime })
                }
            } else if let loadError {
                ContentUnavailableView {
                    Label("Updates Unavailable", systemImage: "exclamationmark.circle")
                } description: {
                    Text(loadError)
                } actions: {
                    Button("Reload Status") { Task { await loadStatus() } }
                }
            } else {
                ProgressView().frame(maxWidth: .infinity, maxHeight: .infinity)
            }
        }
        .task {
            if checkOnOpen, model.operation == nil { await checkOnline() }
            else { await loadStatus() }
        }
        .sheet(isPresented: $configuringChannel) {
            AppUpdateChannelSheet(canClear: snapshot?.items.contains {
                $0.kind == .app && $0.state != .unconfigured
            } == true) {
                configuringChannel = false
                await loadStatus()
            }
        }
        .confirmationDialog("Build a new template release?", isPresented: Binding(
            get: { rebuildTarget != nil }, set: { if !$0 { rebuildTarget = nil } }
        ), titleVisibility: .visible, presenting: rebuildTarget) { item in
            Button("Build Release") {
                guard let reference = item.reference else { return }
                Task { await model.startJob(CoreRequest("jobs.start", id: reference, kind: "build")) }
            }
        } message: { item in
            Text("Devline will fetch current dependencies for \(item.name) and verify the new release offline. Existing releases stay available.")
        }
    }

    private var documentationUpdates: [CoreRequest] {
        []
    }

    private var header: some View {
        HStack(spacing: 12) {
            VStack(alignment: .leading, spacing: 4) {
                Text(checkSummary).font(.headline)
                Text(checking ? "Checking update sources…" : lastCheckLabel)
                    .font(.callout).foregroundStyle(.secondary)
            }
            Spacer(minLength: 8)
            Picker("Filter updates", selection: $filter) {
                Text("All Items").tag(Filter.allItems)
                Text("Available").tag(Filter.available)
            }
            .pickerStyle(.segmented)
            .labelsHidden()
            .fixedSize()
            .accessibilityLabel("Filter updates")
            if checking { ProgressView().controlSize(.small) }
            Button("Check for Updates") { Task { await checkOnline() } }
                .buttonStyle(.borderedProminent)
                .disabled(checking || loading || model.operation != nil)
                .help("Connect to upstream sources to check for updates")
        }
        .padding(18)
    }

    private func availableEmptyState(_ snapshot: UpdateSnapshot) -> some View {
        let incomplete = snapshot.checkedAt == nil || snapshot.items.contains {
            ($0.kind == .template || $0.kind == .runtime) && ($0.state == .unchecked || $0.state == .error)
        }
        return ContentUnavailableView(
            "No Updates Available",
            systemImage: "tray",
            description: Text(incomplete
                ? "Check online to refresh sources that haven’t reported a result."
                : "No updates were reported by the sources checked.")
        )
    }

    private var checkSummary: String {
        guard let snapshot else { return "Update Status" }
        let count = snapshot.items.filter { ($0.kind == .template || $0.kind == .runtime) && $0.state == .available }.count
        if count > 0 { return count == 1 ? "1 Update Available" : "\(count) Updates Available" }
        return "Update Status"
    }

    private var lastCheckLabel: String {
        guard let stamp = snapshot?.checkedAt, stamp > 0, stamp.isFinite else {
            return "Check online to see what’s available."
        }
        let time = Date(timeIntervalSince1970: stamp).formatted(date: .abbreviated, time: .shortened)
        return "Last checked \(time)"
    }

    private func updateList(_ items: [UpdateItem]) -> some View {
        List {
            ForEach(UpdateItem.Kind.allCases) { kind in
                let group = items.filter { $0.kind == kind }.sorted {
                    if $0.state.sortOrder != $1.state.sortOrder { return $0.state.sortOrder < $1.state.sortOrder }
                    return $0.name.localizedStandardCompare($1.name) == .orderedAscending
                }
                if !group.isEmpty {
                    Section {
                        ForEach(group) { item in updateRow(item) }
                    } header: {
                        HStack {
                            Text(kind.title)
                            Spacer()
                            if kind == .app {
                                Button("Configure Channel…") { configuringChannel = true }
                                    .buttonStyle(.borderless)
                                    .disabled(checking || model.operation != nil)
                            }
                        }
                    }
                }
            }
        }
        .listStyle(.inset)
    }

    private func updateRow(_ item: UpdateItem) -> some View {
        HStack(alignment: .center, spacing: 14) {
            VStack(alignment: .leading, spacing: 4) {
                HStack(spacing: 8) {
                    Text(item.name).font(.body.weight(.medium))
                    if let versions = item.versionSummary {
                        Text(versions).font(.caption.monospacedDigit()).foregroundStyle(.secondary)
                    }
                }
                if !item.detail.isEmpty {
                    Text(item.detail)
                        .font(.caption).foregroundStyle(.secondary)
                        .lineLimit(2)
                        .help(item.detail)
                }
            }
            Spacer(minLength: 0)
            VStack(alignment: .trailing, spacing: 5) {
                Label(item.state.label, systemImage: item.state.icon)
                    .font(.caption)
                    .foregroundStyle(statusColor(item.state))
                    .fixedSize()
                action(for: item)
            }
        }
        .padding(.vertical, 5)
    }

    @ViewBuilder private func action(for item: UpdateItem) -> some View {
        if let request = item.docInstallRequest {
            Button("Install Update") {
                apply(request, label: "Installing \(item.name)", completion: "Installed \(item.name).")
            }
            .controlSize(.small)
            .disabled(checking || model.operation != nil)
            .accessibilityLabel("Install \(item.name) update")
        } else if item.action == "install_template", let reference = item.reference {
            Button("Install Update") {
                apply(CoreRequest("templates.install", id: reference),
                      label: "Installing \(item.name)", completion: "Installed \(item.name).")
            }
            .controlSize(.small)
            .disabled(checking || model.operation != nil)
            .accessibilityLabel("Install \(item.name) update")
        } else if item.action == "rebuild_template", item.reference != nil {
            Button("Build Release…") { rebuildTarget = item }
                .controlSize(.small)
                .disabled(checking || model.operation != nil)
                .accessibilityLabel("Build a release of \(item.name)")
        } else if let url = item.externalURL {
            Button {
                if !NSWorkspace.shared.open(url) {
                    loadError = "Could not open the update page."
                }
            } label: {
                Label(linkTitle(item.kind), systemImage: "arrow.up.right")
            }
            .controlSize(.small)
            .accessibilityLabel("Open \(item.name) update page")
        }
    }

    private func statusColor(_ state: UpdateItem.State) -> Color {
        switch state {
        case .available: .accentColor
        case .error: .orange
        default: .secondary
        }
    }

    private func linkTitle(_ kind: UpdateItem.Kind) -> String {
        switch kind {
        case .docs: "Open Source"
        case .runtime: "Open Download Page"
        case .template, .app: "Open Release Page"
        }
    }

    private func loadStatus() async {
        loading = true
        loadError = nil
        defer { loading = false }
        do {
            snapshot = try await Core.request(CoreRequest("updates.status"))
        } catch is CancellationError {
        } catch {
            loadError = error.localizedDescription
        }
    }

    private func checkOnline() async {
        guard !checking, !loading, model.operation == nil else { return }
        checking = true
        loadError = nil
        defer { checking = false }
        do {
            snapshot = try await Core.request(CoreRequest("updates.check"))
        } catch is CancellationError {
        } catch {
            loadError = error.localizedDescription
        }
    }

    private func apply(_ request: CoreRequest, label: String, completion: String) {
        Task {
            _ = await model.mutate([request], label: label, completion: completion)
            await loadStatus()
        }
    }
}

private struct AppUpdateChannelSheet: View {
    @Environment(\.dismiss) private var dismiss
    let canClear: Bool
    let didSave: () async -> Void
    @State private var input = ""
    @State private var saving = false
    @State private var failure: String?
    @FocusState private var fieldFocused: Bool

    private var manifestURL: URL? {
        publicHTTPSURL(input.trimmingCharacters(in: .whitespacesAndNewlines), allowQuery: false)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            VStack(alignment: .leading, spacing: 5) {
                Text("App Update Channel").font(.title2.weight(.semibold))
                Text("Enter a public HTTPS release manifest for this build. Devline uses it the next time you check for updates.")
                    .font(.callout).foregroundStyle(.secondary)
            }
            TextField("https://example.com/devline-release.json", text: $input)
                .textFieldStyle(.roundedBorder)
                .focused($fieldFocused)
                .accessibilityLabel("Release manifest URL")
                .disabled(saving)
            if let failure {
                Label(failure, systemImage: "exclamationmark.circle")
                    .font(.callout).foregroundStyle(.red)
            } else if !input.isEmpty && manifestURL == nil {
                Text("Use an HTTPS URL without credentials, a query, or a fragment.")
                    .font(.caption).foregroundStyle(.secondary)
            }
            HStack {
                if canClear {
                    Button("Clear Channel", role: .destructive) { Task { await save(url: "") } }
                        .disabled(saving)
                }
                Spacer()
                if saving { ProgressView().controlSize(.small) }
                Button("Cancel") { dismiss() }
                    .keyboardShortcut(.cancelAction)
                    .disabled(saving)
                Button("Save Channel") {
                    if let manifestURL { Task { await save(url: manifestURL.absoluteString) } }
                }
                    .keyboardShortcut(.defaultAction)
                    .buttonStyle(.borderedProminent)
                    .disabled(manifestURL == nil || saving)
            }
        }
        .padding(22)
        .frame(width: 510)
        .interactiveDismissDisabled(saving)
        .task { fieldFocused = true }
    }

    private func save(url: String) async {
        guard !saving else { return }
        saving = true
        failure = nil
        defer { saving = false }
        do {
            let _: Acknowledgment = try await Core.request(CoreRequest("updates.configure", url: url))
            await didSave()
        } catch {
            failure = error.localizedDescription
        }
    }
}

private func publicHTTPSURL(_ value: String, allowQuery: Bool = true) -> URL? {
    guard let components = URLComponents(string: value),
          components.scheme?.lowercased() == "https",
          components.host?.isEmpty == false,
          components.user == nil,
          components.password == nil,
          (allowQuery || components.query == nil),
          (allowQuery || components.fragment == nil) else { return nil }
    return components.url
}
