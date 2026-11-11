import Foundation

@main struct ProjectNavigationTests {
    static func main() throws {
        var recent: [RecentProject] = []
        for index in 0..<10 {
            recent = RecentProject.remembering("/tmp/project-\(index)", in: recent, now: Date(timeIntervalSince1970: Double(index)))
        }
        precondition(recent.count == 8 && recent.first?.name == "project-9" && recent.last?.name == "project-2")
        recent = RecentProject.remembering("/tmp/project-4/../project-4", in: recent, now: Date(timeIntervalSince1970: 20))
        precondition(recent.count == 8 && recent.first?.name == "project-4")
        precondition(recent.filter { $0.name == "project-4" }.count == 1)
        let restored = try JSONDecoder().decode([RecentProject].self, from: JSONEncoder().encode(recent))
        precondition(restored == recent)

        // Opening a project moves it to the front of the history, but never moves it in the sidebar.
        let before = RecentProject.listed(recent).map(\.name)
        let reopened = RecentProject.remembering("/tmp/project-7", in: recent, now: Date(timeIntervalSince1970: 30))
        precondition(reopened.first?.name == "project-7")
        precondition(RecentProject.listed(reopened).map(\.name) == before)
        precondition(before == before.sorted { $0.localizedStandardCompare($1) == .orderedAscending })

        // The sidebar highlights the open project however its path was spelled.
        precondition(sameProject("/tmp/project-4/../project-4", "/tmp/project-4"))
        precondition(sameProject("/tmp/project-4", "/tmp/project-4/"))
        precondition(!sameProject("/tmp/project-4", "/tmp/project-5") && !sameProject("/tmp/project-4", nil))

        // A new project starts with a valid folder name taken from its starter.
        precondition(projectSlug(for: "python/fastapi-api") == "fastapi-api")
        precondition(projectSlug(for: "dotnet/ASPNET_MVC") == "aspnet-mvc")
        precondition(projectSlug(for: "node/--react--web--") == "react-web")
        precondition(projectSlug(for: "go/9lives") == "project" && projectSlug(for: "") == "project")

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        func job(_ result: String, state: String = "succeeded") throws -> JobRecord {
            try decoder.decode(JobRecord.self, from: Data("""
            {"id":"job-1","kind":"content_update","state":"\(state)","stage":"Complete","created_at":1,"updated_at":2,"target_path":"/tmp/project","target_reference":"go@1.0","result":\(result)}
            """.utf8))
        }
        let partial = try job(#"{"complete":false,"status":"needs_attention","steps":[{"id":"go","name":"Go docs","state":"failed","detail":"Unavailable offline"}]}"#)
        precondition(partial.needsAttention && partial.stateLabel == "Needs Attention")
        precondition(partial.result?.steps?.first?.needsAttention == true && partial.targetPath == "/tmp/project")
        let incomplete = try job(#"{"ready_offline":false}"#)
        let complete = try job(#"{"complete":true,"status":"complete"}"#)
        let failed = try job("null", state: "failed")
        precondition(incomplete.needsAttention && !complete.needsAttention && failed.needsAttention)
        let unchecked = try job(#"{"all_checked":false,"sources":[{"id":"docs","name":"Official references","state":"error","detail":"Publisher unavailable"}]}"#)
        precondition(unchecked.needsAttention && unchecked.result?.issues.first?.detail == "Publisher unavailable")
        for result in ["null", "[]", "true", #"{"status":true,"complete":"legacy","steps":["legacy"]}"#] {
            _ = try job(result)
        }
        print("Project navigation: recent history deduplication, bounds, persistence and heterogeneous job results passed")
    }
}
