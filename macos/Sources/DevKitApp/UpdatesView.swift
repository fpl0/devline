import AppKit
import SwiftUI

struct UpdateSnapshot: Decodable, Sendable {
    let checkedAt: TimeInterval?
    let items: [UpdateItem]
    let sources: [UpdateSource]?
    let availableContent: Int?
    let allChecked: Bool?
}

struct UpdateSource: Decodable, Identifiable, Sendable {
    let id, name, state, detail: String
    let checkedAt: TimeInterval?
}

struct UpdateItem: Decodable, Identifiable, Sendable {
    enum Kind: String, CaseIterable, Decodable, Identifiable, Sendable {
        case docs, template, image, runtime, app

        var id: String { rawValue }

        var title: String {
            switch self {
            case .docs: "Documentation"
            case .template: "App Starters"
            case .image: "Image Archives"
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
    let delivery: String?
    let pending: Bool?

    var versionSummary: String? {
        if let currentVersion, let availableVersion, currentVersion != availableVersion {
            return "\(currentVersion) → \(availableVersion)"
        }
        if let currentVersion { return currentVersion }
        if let availableVersion { return "Available \(availableVersion)" }
        return nil
    }

    var isContentUpdate: Bool {
        state == .available && reference != nil &&
            ["install_docs", "prepare_local_docs", "install_template", "rebuild_template", "update_image"].contains(action ?? "")
    }

    var docInstallRequest: CoreRequest? {
        guard action == "install_docs", let reference else { return nil }
        let parts = reference.split(separator: "@", maxSplits: 1, omittingEmptySubsequences: false)
        guard let id = parts.first, !id.isEmpty else { return nil }
        let version = parts.count == 2 ? String(parts[1]) : availableVersion
        guard let version, !version.isEmpty else { return nil }
        return CoreRequest("jobs.start", id: String(id), version: version, kind: delivery == "local" ? "docs_prepare" : "docs_install")
    }

    var externalURL: URL? {
        guard action == "open_url", let url else { return nil }
        return publicHTTPSURL(url)
    }
}

struct UpdatesView: View {
    private enum Page: Hashable { case overview, docs, starters, images, app }
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
    @State private var configuringChannel = false
    @State private var page = Page.overview
    private var contentUpdates: [UpdateItem] { snapshot?.items.filter(\.isContentUpdate) ?? [] }
    private var latestContentJob: JobRecord? {
        model.jobs.first { ["content_check", "content_update"].contains($0.kind) }
    }
    private var contentJob: JobRecord? {
        model.jobs.first { $0.isActive && ["content_check", "content_update"].contains($0.kind) }
    }
    private var busy: Bool { checking || loading || contentJob != nil || model.operation != nil }

    var body: some View {
        // Keep tab content within the existing window. Populated update lists
        // should scroll instead of increasing the window's content minimum.
        GeometryReader { viewport in
            TabView(selection: $page) {
                overview.tabItem { Text("Overview") }.tag(Page.overview)
                DocDistributionView(model: model).tabItem { Text("Documentation") }.tag(Page.docs)
                sourceUpdates.tabItem { Text("Starters and Runtimes") }.tag(Page.starters)
                ImageUpdatesView(model: model).tabItem { Text("Infrastructure") }.tag(Page.images)
                Form { AppUpdatesSettings() }.formStyle(.grouped).tabItem { Text("Devline App") }.tag(Page.app)
            }
            .frame(width: viewport.size.width, height: viewport.size.height)
        }.padding(.top, 8)
        .task {
            await loadStatus()
            if checkOnOpen { await checkOnline() }
        }
        .onChange(of: model.jobs.filter { !$0.isActive }.map(\.id)) { _, _ in
            Task { await loadStatus() }
        }
    }

    private var overview: some View {
        VStack(spacing: 0) {
            VStack(alignment: .leading, spacing: 14) {
                HStack(alignment: .top, spacing: 18) {
                    VStack(alignment: .leading, spacing: 5) {
                        Text("Keep your toolbox current").font(.title2.weight(.semibold))
                        Text("Update the docs, starters, and image archives already in your library. Previous releases and project pins stay available.")
                            .font(.callout).foregroundStyle(.secondary)
                            .fixedSize(horizontal: false, vertical: true)
                    }
                    Spacer(minLength: 8)
                    Button("Check All Sources") { Task { await checkOnline() } }.disabled(busy)
                    Button("Update Available (\(contentUpdates.count))") {
                        Task { await model.startJob(CoreRequest("jobs.start", kind: "content_update"), inspect: false) }
                    }.buttonStyle(.borderedProminent).disabled(busy || contentUpdates.isEmpty)
                }
                if let job = contentJob {
                    HStack(spacing: 10) {
                        ProgressView().controlSize(.small)
                        Text(job.stage).font(.callout)
                        Spacer()
                        Button("View Progress") { model.inspectedJob = job }
                    }
                } else if let job = latestContentJob, job.needsAttention || job.state == "cancelled" {
                    HStack(alignment: .top, spacing: 10) {
                        Image(systemName: job.icon).foregroundStyle(.orange)
                        VStack(alignment: .leading, spacing: 4) {
                            Text("\(job.title) · \(job.stateLabel)").font(.callout.weight(.medium))
                            Text(job.result?.issues.first?.detail ?? job.error ?? "Open the result to see what still needs attention. Completed updates were kept.")
                                .font(.caption).foregroundStyle(.secondary).lineLimit(3)
                        }
                        Spacer()
                        Button("View Result") { model.inspectedJob = job }
                        Button(job.kind == "content_check" ? "Retry Checks" : "Retry Updates") {
                            Task {
                                if job.kind == "content_check" { await checkOnline() }
                                else { await model.startJob(CoreRequest("jobs.start", kind: "content_update"), inspect: false) }
                            }
                        }.disabled(busy)
                    }
                } else if let loadError {
                    Label(loadError, systemImage: "exclamationmark.circle").font(.callout).foregroundStyle(.orange)
                } else {
                    Text("Checks use the internet. Updates may download packages and build new starter releases; your projects keep their selected versions.")
                        .font(.caption).foregroundStyle(.secondary)
                }
            }.padding(20)
            Divider()
            List {
                Section("Available content updates") {
                    if contentUpdates.isEmpty {
                        VStack(alignment: .leading, spacing: 6) {
                            Label(snapshot?.allChecked == true ? "Your prepared content is current" : "No confirmed content updates yet", systemImage: snapshot?.allChecked == true ? "checkmark.circle" : "clock")
                                .font(.headline)
                            Text(snapshot?.allChecked == true ? "Check again when you want to look for newer releases." : "Check all sources to refresh missing or failed results. Your saved library remains usable offline.")
                                .font(.callout).foregroundStyle(.secondary)
                        }.padding(.vertical, 8)
                    } else {
                        ForEach(contentUpdates) { item in updateRow(item) }
                    }
                }
                Section("Sources and freshness") {
                    ForEach(snapshot?.sources ?? []) { source in
                        HStack(alignment: .top, spacing: 12) {
                            Image(systemName: source.state == "current" ? "checkmark.circle" : source.state == "error" ? "exclamationmark.circle" : "clock")
                                .foregroundStyle(source.state == "error" ? Color.orange : .secondary)
                            VStack(alignment: .leading, spacing: 4) {
                                Text(source.name).font(.body.weight(.medium))
                                Text(source.detail).font(.caption).foregroundStyle(.secondary)
                                if let checked = source.checkedAt {
                                    Text("Last checked \(Date(timeIntervalSince1970: checked).formatted(date: .abbreviated, time: .shortened))")
                                        .font(.caption).foregroundStyle(.secondary)
                                }
                            }
                            Spacer()
                            if source.state != "current" {
                                Button(source.state == "error" ? "Retry Checks" : "Check Sources") { Task { await checkOnline() } }
                                    .controlSize(.small).disabled(busy)
                            }
                        }.padding(.vertical, 5)
                    }
                }
                Section("Managed separately") {
                    HStack {
                        VStack(alignment: .leading, spacing: 5) {
                            Text("Local toolchains").font(.headline)
                            Text("Devline detects installed runtimes and reports publisher releases. Use your runtime manager or installer to upgrade them.")
                                .font(.caption).foregroundStyle(.secondary)
                        }
                        Spacer()
                        Button("View Toolchain Updates") { page = .starters }
                    }.padding(.vertical, 5)
                    HStack {
                        VStack(alignment: .leading, spacing: 5) {
                            Text("Devline app").font(.headline)
                            Text("App releases are separate from your content library.").font(.caption).foregroundStyle(.secondary)
                        }
                        Spacer()
                        Button("View App Release") { page = .app }
                    }.padding(.vertical, 5)
                }
            }.listStyle(.inset)
                .frame(minHeight: 0, maxHeight: .infinity)
        }.frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }

    private var sourceUpdates: some View {
        VStack(spacing: 0) {
            header
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
        .sheet(isPresented: $configuringChannel) {
            AppUpdateChannelSheet(canClear: snapshot?.items.contains {
                $0.kind == .app && $0.state != .unconfigured
            } == true) {
                configuringChannel = false
                await loadStatus()
            }
        }
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
                .disabled(busy)
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
                                    .disabled(busy)
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
                Task { await model.startJob(request) }
            }
            .controlSize(.small)
            .disabled(busy)
            .accessibilityLabel("Install \(item.name) update")
        } else if item.action == "prepare_local_docs", let reference = item.reference {
            Button("Refresh from Xcode") {
                Task { await model.startJob(CoreRequest("jobs.start", id: reference, kind: "docs_prepare")) }
            }.controlSize(.small).disabled(busy)
        } else if item.action == "update_image", let reference = item.reference {
            Button("Update Image") {
                Task { await model.startJob(CoreRequest("jobs.start", id: reference, kind: "infra_update")) }
            }.controlSize(.small).disabled(busy)
        } else if item.action == "install_template", item.reference != nil {
            Button(item.pending == true ? "Finish Update" : "Install Update") {
                Task { await model.startJob(CoreRequest("jobs.start", id: item.id, kind: "content_update"), inspect: false) }
            }
            .controlSize(.small)
            .disabled(busy)
            .accessibilityLabel("Install \(item.name) update")
        } else if item.action == "rebuild_template", item.reference != nil {
            Button("Build Release") {
                if let reference = item.reference {
                    Task { await model.startJob(CoreRequest("jobs.start", id: reference, kind: "build")) }
                }
            }
                .controlSize(.small)
                .disabled(busy)
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
        case .template, .image, .app: "Open Release Page"
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
        guard !busy else { return }
        checking = true
        loadError = nil
        defer { checking = false }
        await model.startJob(CoreRequest("jobs.start", kind: "content_check"), inspect: false)
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
