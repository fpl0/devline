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
    @Published private(set) var recentProjects: [RecentProject] = {
        guard let data = UserDefaults.standard.data(forKey: "devline.recentProjects") else { return [] }
        return (try? JSONDecoder().decode([RecentProject].self, from: data)) ?? []
    }()

    var completedJobIDs: [String] { jobs.filter { !$0.isActive }.map(\.id) }

    func rememberProject(_ path: String) {
        recentProjects = RecentProject.remembering(path, in: recentProjects)
        saveRecentProjects()
    }
    func forgetProject(_ path: String) {
        recentProjects.removeAll { $0.path == path }
        saveRecentProjects()
    }
    func clearRecentProjects() { recentProjects = []; saveRecentProjects() }
    private func saveRecentProjects() {
        if let data = try? JSONEncoder().encode(recentProjects) {
            UserDefaults.standard.set(data, forKey: "devline.recentProjects")
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
                jobs = updated
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
        loadingCatalog = true
        defer { if catalogReadID == readID { loadingCatalog = false } }
        do {
            let updated: Catalog = try await Core.request(CoreRequest("catalog"))
            guard readID > catalogResultID else { return }
            catalogResultID = readID
            let previous = catalog?.docs.filter(\.installed).map { "\($0.reference):\($0.active == true)" }
            let next = updated.docs.filter(\.installed).map { "\($0.reference):\($0.active == true)" }
            catalog = updated
            catalogError = nil
            if previous != next { libraryRevision += 1 }
        } catch is CancellationError { throw CancellationError() }
        catch {
            guard readID > catalogResultID else { return }
            catalogResultID = readID
            catalogError = error.localizedDescription
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
