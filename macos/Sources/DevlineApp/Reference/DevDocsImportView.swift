import SwiftUI

private struct DevDocsCatalog: Decodable, Sendable {
    struct Snapshot: Decodable, Identifiable, Sendable {
        let slug, name: String
        let release, version, sourceUrl: String?
        let mtime, dbSize: Int64?

        var id: String { slug }
        var releaseName: String {
            [release, version].compactMap { $0 }.first { !$0.isEmpty } ?? "Snapshot"
        }
    }

    let docsets: [Snapshot]
}

struct DevDocsImportView: View {
    @ObservedObject var model: WorkspaceModel
    let close: () -> Void
    @State private var catalog: DevDocsCatalog?
    @State private var query = ""
    @State private var family: String?
    @State private var slug: String?
    @State private var loading = false
    @State private var starting = false
    @State private var cancelling = false
    @State private var failure: String?

    private var snapshotsByName: [String: [DevDocsCatalog.Snapshot]] {
        Dictionary(grouping: catalog?.docsets ?? [], by: \.name)
    }

    private var visibleNames: [String] {
        snapshotsByName.keys.filter { query.isEmpty || $0.localizedCaseInsensitiveContains(query) }
            .sorted { $0.localizedStandardCompare($1) == .orderedAscending }
    }

    private var snapshots: [DevDocsCatalog.Snapshot] { snapshotsByName[family ?? ""] ?? [] }
    private var selected: DevDocsCatalog.Snapshot? { snapshots.first { $0.slug == slug } }
    private var jobs: [JobRecord] { model.jobs.filter { $0.kind == "docs_import_devdocs" } }
    private var activeJob: JobRecord? { jobs.first(where: \.isActive) }
    private var selectedJob: JobRecord? { jobs.first { $0.targetReference == slug } }

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 14) {
                Button(action: close) { Label("Back to Manage Docs", systemImage: "chevron.left") }
                    .buttonStyle(.borderless)
                Spacer()
                if loading { ProgressView().controlSize(.small) }
                Button("Refresh Catalog") { Task { await load() } }.disabled(loading)
            }.padding(14)
            Divider()
            if catalog != nil {
                HStack(spacing: 0) {
                    VStack(spacing: 8) {
                        TextField("Find a docset", text: $query).textFieldStyle(.roundedBorder)
                            .padding(.horizontal, 12).padding(.top, 10)
                        List(visibleNames, id: \.self, selection: $family) { name in
                            Text(name).tag(name)
                        }.listStyle(.inset)
                    }.frame(width: 235)
                    Divider()
                    if let selected {
                        snapshotDetails(selected)
                    } else {
                        ContentUnavailableView("Choose a Docset", systemImage: "books.vertical",
                                               description: Text("Select a release to download it for offline reading."))
                    }
                }
            } else if loading {
                ProgressView("Loading DevDocs catalog…").frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ContentUnavailableView {
                    Label("DevDocs Catalog Unavailable", systemImage: "wifi.slash")
                } description: {
                    Text(failure ?? "Try again when you’re online.")
                } actions: {
                    Button("Retry") { Task { await load() } }
                }
            }
            if let job = activeJob ?? selectedJob { progress(job) }
        }
        .task { await load() }
        .onChange(of: family) { _, _ in slug = snapshots.first?.slug }
        .onChange(of: query) { _, _ in
            if let family, !visibleNames.contains(family) { self.family = nil }
        }
        .onChange(of: activeJob?.id) { _, _ in cancelling = false }
    }

    private func snapshotDetails(_ snapshot: DevDocsCatalog.Snapshot) -> some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 15) {
                Text(snapshot.name).font(.title2.weight(.semibold)).accessibilityAddTraits(.isHeader)
                Picker("Release", selection: $slug) {
                    ForEach(snapshots) { item in
                        Text(item.releaseName).tag(Optional(item.slug))
                    }
                }
                LabeledContent("Snapshot Date", value: snapshot.mtime.map {
                    Date(timeIntervalSince1970: Double($0)).formatted(date: .abbreviated, time: .omitted)
                } ?? "Unavailable")
                LabeledContent("Estimated Database Size", value: snapshot.dbSize.map(storageSize) ?? "Unavailable")
                if let raw = snapshot.sourceUrl, let source = safeExternalURL(raw) {
                    Link("Publisher Source", destination: source)
                }
                Text("Imports from DevDocs are saved on this Mac. Find them in Docs under Imported.")
                    .font(.callout).foregroundStyle(.secondary)
                if let failure { Label(failure, systemImage: "exclamationmark.circle").foregroundStyle(.orange) }
                HStack {
                    Button(selectedJob?.needsAttention == true || selectedJob?.state == "cancelled" ? "Retry Import" : "Download & Import") {
                        Task { await start(snapshot) }
                    }.buttonStyle(.borderedProminent).disabled(starting || activeJob != nil)
                    if selectedJob?.state == "succeeded" {
                        Button("Show Imported Docs", action: close)
                    }
                }
            }.padding(20).frame(maxWidth: .infinity, alignment: .leading)
        }.frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private func progress(_ job: JobRecord) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            Divider()
            HStack(spacing: 10) {
                if job.isActive { ProgressView().controlSize(.small) }
                Text(job.isActive ? job.stage : job.error ?? job.stage)
                    .lineLimit(2)
                Spacer()
                if job.isActive {
                    Button("Cancel Import") { Task { await cancel(job) } }.disabled(cancelling)
                }
            }.padding(.horizontal, 14).padding(.bottom, 8)
            if job.isActive, let completed = job.completedBytes, let total = job.totalBytes, total > 0 {
                ProgressView(value: Double(min(completed, total)), total: Double(total))
                    .padding(.horizontal, 14).padding(.bottom, 8)
            }
        }.font(.callout)
    }

    private func load() async {
        guard !loading else { return }
        loading = true
        failure = nil
        defer { loading = false }
        do {
            catalog = try await Core.request(CoreRequest("docs.devdocs.catalog"))
            if family == nil { family = visibleNames.first }
            if !snapshots.contains(where: { $0.slug == slug }) { slug = snapshots.first?.slug }
        } catch is CancellationError {}
        catch { failure = error.localizedDescription }
    }

    private func start(_ snapshot: DevDocsCatalog.Snapshot) async {
        guard !starting, activeJob == nil else { return }
        starting = true
        failure = nil
        defer { starting = false }
        do {
            let job: JobRecord = try await Core.request(CoreRequest("jobs.start", id: snapshot.slug, kind: "docs_import_devdocs"))
            await model.refreshJobs(startedJobID: job.id)
        } catch is CancellationError {}
        catch { failure = error.localizedDescription }
    }

    private func cancel(_ job: JobRecord) async {
        cancelling = true
        do { let _: Acknowledgment = try await Core.request(CoreRequest("jobs.cancel", jobId: job.id)) }
        catch { failure = error.localizedDescription; cancelling = false }
    }
}
