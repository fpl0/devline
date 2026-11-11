import SwiftUI

struct UpdateSnapshot: Decodable, Sendable {
    let checkedAt: TimeInterval?
    let items: [UpdateItem]
    let sources: [UpdateSource]?
    let allChecked: Bool?
}

struct UpdateSource: Decodable, Identifiable, Sendable {
    let id, name, state, detail: String
    let checkedAt: TimeInterval?
}

struct UpdateItem: Decodable, Equatable, Identifiable, Sendable {
    enum Kind: String, Decodable, Sendable {
        case docs, template, image, runtime, app

        var title: String {
            switch self {
            case .docs: "Documentation"
            case .template: "App Starters"
            case .image: "Service Images"
            case .runtime: "Toolchains"
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
            case .unchecked: "Not Checked"
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
        let current = currentVersion.map { kind == .docs ? displayDocVersion($0) : $0 }
        let available = availableVersion.map { kind == .docs ? displayDocVersion($0) : $0 }
        if let current, let available, current != available { return "\(current) → \(available)" }
        return current ?? available
    }

    var canUpdate: Bool {
        state == .available && reference != nil &&
            ["install_docs", "prepare_local_docs", "install_template", "rebuild_template", "update_image"].contains(action ?? "")
    }

    var updateMethod: String? {
        switch action {
        case "prepare_local_docs": "From the selected Xcode SDK"
        case "rebuild_template": "Builds and verifies a new starter release"
        case "install_docs" where delivery == "local", "install_template": "Included with Devline"
        default: nil
        }
    }

    var externalURL: URL? {
        guard action == "open_url", let url,
              let safe = safeExternalURL(url), safe.scheme?.lowercased() == "https" else { return nil }
        return safe
    }
}

struct UpdatesView: View {
    private struct StatusRevision: Equatable {
        let library: Int
        let finishedJobs: [String]
        let retry: Int
    }

    @ObservedObject var model: WorkspaceModel
    @State private var snapshot: UpdateSnapshot?
    @State private var loadedRevision: StatusRevision?
    @State private var retry = 0
    @State private var loadError: String?
    @State private var submitting = false
    @State private var showingSources = false

    private var statusRevision: StatusRevision {
        StatusRevision(library: model.libraryRevision,
                       finishedJobs: model.completedJobIDs, retry: retry)
    }

    private var loading: Bool { loadedRevision != statusRevision }
    private var updates: [UpdateItem] { snapshot?.items.filter { $0.canUpdate && $0.kind != .docs } ?? [] }
    private var jobs: [JobRecord] { model.jobs.filter { $0.kind == "content_check" || $0.kind == "content_update" } }
    private var activeJob: JobRecord? { jobs.first(where: \.isActive) }
    private var busy: Bool { loading || submitting || activeJob != nil || model.operation != nil }
    private var sources: [UpdateSource] { snapshot?.sources?.filter { $0.id != "docs" && $0.id != "local" } ?? [] }
    private var allChecked: Bool { !sources.isEmpty && sources.allSatisfy { $0.state == "current" } }
    private var checkFailed: Bool { sources.contains { $0.state == "error" } }

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            statusBanner
            if !updates.isEmpty {
                updateList
            } else {
                emptyState
            }
            if !sources.isEmpty && (checkFailed || sources.contains { $0.checkedAt != nil }) {
                Divider()
                checkDetails
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
        .task(id: statusRevision) { [revision = statusRevision] in await loadStatus(revision) }
    }

    private var header: some View {
        HStack(alignment: .center, spacing: 12) {
            VStack(alignment: .leading, spacing: 4) {
                Text("Library Updates").font(.title2.weight(.semibold)).accessibilityAddTraits(.isHeader)
                Text("New releases for starters and saved service images.")
                    .font(.callout).foregroundStyle(.secondary)
            }
            Spacer(minLength: 12)
            if updates.isEmpty { checkButton.buttonStyle(.borderedProminent) }
            else { checkButton.buttonStyle(.bordered) }
            if updates.count > 1 {
                Button("Update All") { Task { await run("content_update") } }
                    .buttonStyle(.borderedProminent)
                    .fixedSize()
                    .disabled(busy || loadError != nil)
            }
        }
        .padding(20)
    }

    private var checkButton: some View {
        Button("Check for Updates") { Task { await run("content_check") } }
            .fixedSize()
            .disabled(busy)
            .help("Look for newer releases online")
    }

    @ViewBuilder private var statusBanner: some View {
        if let job = activeJob {
            banner {
                ProgressView().controlSize(.small)
                VStack(alignment: .leading, spacing: 3) {
                    Text(job.kind == "content_check" ? "Checking for updates…" : "Updating your library…")
                        .fontWeight(.medium)
                    Text(job.stage).font(.caption).foregroundStyle(.secondary)
                }
                Spacer()
                Button("Details…") { model.inspectedJob = job }
            }
        } else if let loadError, snapshot != nil {
            banner {
                Label("Couldn’t refresh update status.", systemImage: "exclamationmark.circle")
                    .help(loadError)
                Spacer()
                Button("Reload") { retry += 1 }
            }
        } else if !loading, let job = jobs.first,
                  (job.state == "cancelled" || job.needsAttention),
                  !(job.kind == "content_check" && checkFailed && job.result?.allChecked == false) {
            banner {
                Label(jobMessage(job), systemImage: job.icon)
                Spacer()
                Button("Details…") { model.inspectedJob = job }
            }
        } else if !loading, let job = jobs.first, job.kind == "content_update", job.state == "succeeded",
                  (job.result?.updated ?? 0) > 0 {
            banner {
                Label("Library updated.", systemImage: "checkmark.circle")
                Spacer()
            }
        }
    }

    private func banner<Content: View>(@ViewBuilder content: () -> Content) -> some View {
        VStack(spacing: 0) {
            HStack(spacing: 10, content: content)
                .font(.callout)
                .padding(.horizontal, 20).padding(.vertical, 10)
                .background(.quaternary.opacity(0.35))
            Divider()
        }
    }

    private func jobMessage(_ job: JobRecord) -> String {
        if job.state == "cancelled" {
            return job.kind == "content_check" ? "Update check cancelled." : "Update cancelled. Completed work was kept."
        }
        return job.kind == "content_check" ? "The update check didn’t finish." : "Some updates couldn’t be applied."
    }

    private var updateList: some View {
        VStack(spacing: 0) {
            VStack(alignment: .leading, spacing: 4) {
                Text(updates.count == 1 ? "1 update available" : "\(updates.count) updates available")
                    .font(.headline).accessibilityAddTraits(.isHeader)
                Text("Your projects keep their pinned versions.")
                    .font(.callout).foregroundStyle(.secondary)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.horizontal, 20).padding(.top, 16).padding(.bottom, 6)

            ScrollView {
                LazyVStack(alignment: .leading, spacing: 0) {
                    ForEach([UpdateItem.Kind.template, .image], id: \.self) { kind in
                        let group = updates.filter { $0.kind == kind }.sorted {
                            $0.name.localizedStandardCompare($1.name) == .orderedAscending
                        }
                        if !group.isEmpty {
                            Text(kind.title)
                                .font(.subheadline.weight(.semibold)).foregroundStyle(.secondary)
                                .accessibilityAddTraits(.isHeader)
                                .padding(.top, 14).padding(.bottom, 7)
                            Divider()
                            ForEach(group) { item in
                                updateRow(item)
                                if item.id != group.last?.id { Divider() }
                            }
                        }
                    }
                }
                .padding(.horizontal, 20).padding(.bottom, 14)
            }
            .frame(maxHeight: .infinity)
        }
    }

    private func updateRow(_ item: UpdateItem) -> some View {
        HStack(spacing: 14) {
            VStack(alignment: .leading, spacing: 4) {
                Text(item.name).font(.body.weight(.medium))
                if let summary = item.versionSummary {
                    Text(summary).font(.callout.monospacedDigit()).foregroundStyle(.secondary)
                }
                if let method = item.updateMethod {
                    Text(method).font(.caption).foregroundStyle(.secondary)
                }
            }
            Spacer(minLength: 8)
            if updates.count == 1 { updateButton(item).buttonStyle(.borderedProminent) }
            else { updateButton(item).buttonStyle(.bordered) }
        }
        .padding(.vertical, 10)
        .help(item.pending == true ? item.detail : "")
    }

    private func updateButton(_ item: UpdateItem) -> some View {
        Button(item.pending == true ? "Retry" : "Update") {
            if let reference = item.reference { Task { await run("content_update", id: reference) } }
        }
        .disabled(busy || loadError != nil)
        .accessibilityLabel(item.pending == true ? "Retry \(item.name) update" : "Update \(item.name)")
    }

    @ViewBuilder private var emptyState: some View {
        if loading && snapshot == nil {
            ProgressView("Loading update status…").frame(maxWidth: .infinity, maxHeight: .infinity)
        } else if let loadError, snapshot == nil {
            ContentUnavailableView {
                Label("Update Status Unavailable", systemImage: "exclamationmark.circle")
            } description: {
                Text(loadError).textSelection(.enabled)
            } actions: {
                Button("Reload") { retry += 1 }
            }
        } else if activeJob != nil || submitting {
            Spacer(minLength: 0)
        } else if allChecked {
            ContentUnavailableView("No Library Updates Found", systemImage: "checkmark.circle",
                                   description: Text("You can check again whenever you’re online."))
        } else if checkFailed {
            ContentUnavailableView("No Confirmed Updates", systemImage: "arrow.clockwise",
                                   description: Text("Your saved library is still available offline."))
        } else {
            ContentUnavailableView("Ready to Check", systemImage: "arrow.down.circle",
                                   description: Text("Check online for newer versions of the items in your library."))
        }
    }

    private var checkDetails: some View {
        VStack(alignment: .leading, spacing: 0) {
            if showingSources {
                ScrollView {
                    VStack(alignment: .leading, spacing: 10) {
                        ForEach(sources) { source in
                            VStack(alignment: .leading, spacing: 3) {
                                Text(source.name).font(.callout.weight(.medium))
                                if source.state != "current" {
                                    Text(source.detail).font(.caption).foregroundStyle(.secondary)
                                        .textSelection(.enabled)
                                }
                                if let date = checkDate(source.checkedAt) {
                                    Text(date).font(.caption).foregroundStyle(.secondary)
                                }
                            }
                            .frame(maxWidth: .infinity, alignment: .leading)
                        }
                    }
                    .padding(.horizontal, 20).padding(.top, 12).padding(.bottom, 4)
                }
                .frame(height: 145)
            }
            HStack {
                if checkFailed {
                    Label("Some checks failed", systemImage: "exclamationmark.circle").foregroundStyle(.orange)
                } else if allChecked, let date = checkDate(sources.compactMap(\.checkedAt).min()) {
                    Text("Last checked \(date)").foregroundStyle(.secondary)
                } else {
                    Text("Check incomplete").foregroundStyle(.secondary)
                }
                Spacer()
                Button(showingSources ? "Hide Check Details" : "Check Details…") { showingSources.toggle() }
                    .buttonStyle(.borderless)
            }
            .font(.caption).padding(.horizontal, 20).padding(.vertical, 10)
        }
    }

    private func checkDate(_ stamp: TimeInterval?) -> String? {
        guard let stamp, stamp.isFinite, stamp > 0 else { return nil }
        return Date(timeIntervalSince1970: stamp).formatted(date: .abbreviated, time: .shortened)
    }

    private func loadStatus(_ revision: StatusRevision) async {
        do {
            snapshot = try await Core.request(CoreRequest("updates.status", category: "assets"))
            loadError = nil
        } catch is CancellationError {
            return
        } catch {
            loadError = error.localizedDescription
        }
        loadedRevision = revision
    }

    private func run(_ kind: String, id: String? = nil) async {
        guard !busy else { return }
        submitting = true
        defer { submitting = false }
        if await model.startJob(CoreRequest("jobs.start", id: id, category: "assets", kind: kind,
                                            confirmedOnly: kind == "content_update" ? true : nil), inspect: false) == false {
            retry += 1
        }
    }
}
