import Combine
import Foundation

@MainActor
final class WorkspaceModel: ObservableObject {
    @Published private(set) var catalog: Catalog?
    @Published private(set) var catalogError: String?
    @Published private(set) var loadingCatalog = false
    @Published private(set) var libraryRevision = 0
    @Published private(set) var operation: String?
    @Published var error: String?
    @Published var notice: String?
    @Published private(set) var jobs: [JobRecord] = []
    @Published var inspectedJob: JobRecord?
    private var catalogReadID = 0
    private var catalogResultID = 0
    private var jobMonitor: Task<Void, Never>?
    private var jobRefresh: Task<Void, Error>?
    private var jobRefreshPending = false
    private var loadedJobs = false
    private var jobsStartedBeforeInitialRefresh = Set<String>()
    private var catalogRefreshPending = false
    private var pendingServicePreparations: [String: String] =
        UserDefaults.standard.dictionary(forKey: "devline.pendingServicePreparations") as? [String: String] ?? [:]
    /// The projects listed in the sidebar, which sorts them by name.
    @Published private(set) var recentProjects = RecentProject.restore(
        UserDefaults.standard.data(forKey: RecentProject.storageKey)
    )
    /// The same projects, most recently opened first, for the Open Recent menu.
    private var recentOrder = RecentProject.restore(UserDefaults.standard.data(forKey: RecentProject.storageKey))

    /// Things downloaded one after another so a slow connection is not split many ways.
    enum Download: String { case images, packages }
    /// What is still waiting in each "download all" run, the one in progress first.
    @Published private(set) var downloads: [Download: [String]] = [:]
    private var downloadTasks: [Download: Task<Void, Never>] = [:]
    var imageQueue: [String] { downloads[.images] ?? [] }
    var packageQueue: [String] { downloads[.packages] ?? [] }

    var completedJobIDs: [String] { jobs.filter { !$0.isActive }.map(\.id) }

    /// Service images are named by release (`postgres@18.6`); package shelves by runtime (`python`).
    func download(_ kind: Download, _ items: [String]) {
        guard downloadTasks[kind] == nil, !items.isEmpty else { return }
        downloads[kind] = items
        downloadTasks[kind] = Task {
            defer { downloadTasks[kind] = nil; downloads[kind] = nil }
            var failures: [String] = []
            var reason: String?
            for item in items {
                guard !Task.isCancelled else { return }
                let request = kind == .images
                    ? CoreRequest("jobs.start", id: item, kind: "infra_prepare")
                    : CoreRequest("jobs.start", id: item, kind: "packages_prepare", timeoutSeconds: 7200)
                let started: JobRecord
                do { started = try await Core.request(request) }
                catch { self.error = error.localizedDescription; return }
                await refreshJobs(startedJobID: started.id)
                var finished = jobs.first { $0.id == started.id }
                while finished?.isActive != false {
                    do { try await Task.sleep(for: .seconds(1)) } catch {
                        let _: Acknowledgment? = try? await Core.request(CoreRequest("jobs.cancel", jobId: started.id))
                        return
                    }
                    await refreshJobs()
                    finished = jobs.first { $0.id == started.id }
                }
                if finished?.state != "succeeded" {
                    failures.append(item)
                    reason = reason ?? finished?.error
                }
                downloads[kind]?.removeAll { $0 == item }
            }
            let noun = kind == .images ? "service image" : "package shelf"
            if failures.isEmpty {
                notice = "Saved \(items.count) \(noun)\(items.count == 1 ? "" : kind == .images ? "s" : "es") for offline use."
            } else {
                self.error = "Couldn’t download \(failures.joined(separator: ", ")). \(reason ?? "Check your connection, then try again.")"
            }
        }
    }

    func stopDownloads(_ kind: Download) { downloadTasks[kind]?.cancel() }
    func downloadImages(_ references: [String]) { download(.images, references) }
    func stopImageDownloads() { stopDownloads(.images) }

    func rememberProject(_ path: String) {
        let updated = RecentProject.remembering(path, in: recentOrder)
        guard updated.map(\.path) != recentOrder.map(\.path) else { return }
        recentOrder = updated
        // Reopening a listed project only reorders the Open Recent menu. The sidebar does not
        // change, so the pages that watch this model are not rebuilt for it.
        if Set(updated.map(\.path)) != Set(recentProjects.map(\.path)) { recentProjects = updated }
        saveRecentProjects()
    }
    func forgetProject(_ path: String) {
        recentOrder.removeAll { $0.path == path }
        recentProjects.removeAll { $0.path == path }
        saveRecentProjects()
    }
    func clearRecentProjects() { recentOrder = []; recentProjects = []; saveRecentProjects() }
    private func saveRecentProjects() {
        if let data = try? JSONEncoder().encode(recentOrder) {
            PreferenceBox.box(RecentProject.storageKey).set(data)
        }
    }

    func refresh() async {
        do { try await reload(); await refreshJobs() }
        catch is CancellationError {}
        catch { if catalog != nil { self.error = error.localizedDescription } }
    }

    @discardableResult
    func startJob(_ request: CoreRequest, inspect: Bool = true) async -> Bool {
        do {
            let job: JobRecord = try await Core.request(request)
            if job.kind == "infra_stack_add", let path = job.targetPath {
                pendingServicePreparations[job.id] = path
                UserDefaults.standard.set(pendingServicePreparations, forKey: "devline.pendingServicePreparations")
            }
            if inspect { inspectedJob = job }
            await refreshJobs(startedJobID: job.id)
            return true
        } catch { self.error = error.localizedDescription; return false }
    }

    func refreshJobs(startedJobID: String? = nil) async {
        if !loadedJobs, let startedJobID { jobsStartedBeforeInitialRefresh.insert(startedJobID) }
        do { try await refreshJobSnapshot() }
        catch { return }
        guard jobMonitor == nil, jobs.contains(where: \.isActive) else { return }
        jobMonitor = Task {
            defer { jobMonitor = nil }
            while jobs.contains(where: \.isActive) {
                do {
                    try await Task.sleep(for: .seconds(1))
                    try await refreshJobSnapshot()
                } catch is CancellationError {
                    return
                } catch { self.error = error.localizedDescription; return }
            }
        }
    }

    private func refreshJobSnapshot() async throws {
        jobRefreshPending = true
        if let jobRefresh { return try await jobRefresh.value }
        let task = Task {
            defer { jobRefresh = nil }
            repeat {
                jobRefreshPending = false
                let updated: [JobRecord] = try await Core.request(CoreRequest("jobs.list"))
                let completed = Set(completedJobIDs)
                if updated.contains(where: { !$0.isActive && !completed.contains($0.id)
                    && (loadedJobs || jobsStartedBeforeInitialRefresh.contains($0.id)) }) {
                    catalogRefreshPending = true
                }
                if jobs != updated { jobs = updated }
                loadedJobs = true
                jobsStartedBeforeInitialRefresh.removeAll()
                await continueServicePreparation()
                if catalogRefreshPending {
                    try await reload()
                    catalogRefreshPending = false
                }
            } while jobRefreshPending
        }
        jobRefresh = task
        try await task.value
    }

    private func continueServicePreparation() async {
        for job in jobs where !job.isActive {
            guard let path = pendingServicePreparations[job.id] else { continue }
            if job.state == "succeeded", !job.needsAttention {
                do {
                    let _: JobRecord = try await Core.request(CoreRequest("jobs.start", path: path, kind: "toolbox_prepare"))
                    jobRefreshPending = true
                } catch { self.error = error.localizedDescription; continue }
            }
            pendingServicePreparations.removeValue(forKey: job.id)
            UserDefaults.standard.set(pendingServicePreparations, forKey: "devline.pendingServicePreparations")
        }
    }

    @discardableResult
    func mutate(_ requests: [CoreRequest], label: String, completion: String) async -> Bool {
        guard operation == nil else { return false }
        operation = label
        notice = nil
        defer { operation = nil }

        do {
            for (index, request) in requests.enumerated() {
                if requests.count > 1 { operation = "\(label) (\(index + 1) of \(requests.count))" }
                let _: Acknowledgment = try await Core.request(request)
            }
            try await reload()
            notice = completion
            return true
        } catch {
            try? await reload()
            self.error = error.localizedDescription
            return false
        }
    }

    private func reload() async throws {
        catalogReadID += 1
        let readID = catalogReadID
        if catalog == nil && !loadingCatalog { loadingCatalog = true }
        defer { if catalogReadID == readID && loadingCatalog { loadingCatalog = false } }
        do {
            let updated: Catalog = try await Core.request(CoreRequest("catalog"))
            guard readID > catalogResultID else { return }
            catalogResultID = readID
            if catalogError != nil { catalogError = nil }
            if catalog != updated {
                let previous = catalog?.docs.filter(\.installed).map { "\($0.reference):\($0.active == true)" }
                let next = updated.docs.filter(\.installed).map { "\($0.reference):\($0.active == true)" }
                catalog = updated
                if previous != next { libraryRevision += 1 }
            }
        } catch is CancellationError { throw CancellationError() }
        catch {
            guard readID > catalogResultID else { return }
            catalogResultID = readID
            let message = error.localizedDescription
            if catalogError != message { catalogError = message }
            throw error
        }
    }

    func purgeOldVersions() async throws -> LibraryPurgeResult {
        operation = "Purging older versions"
        let previousRevision = libraryRevision
        defer {
            if libraryRevision == previousRevision { libraryRevision += 1 }
            operation = nil
        }
        let result: LibraryPurgeResult
        do {
            result = try await Core.request(CoreRequest("library.purge"))
        } catch {
            try? await reload()
            throw error
        }
        try await reload()
        return result
    }
}
