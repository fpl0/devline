import Foundation

struct OfflineStatus: Decodable, Sendable {
    struct Doc: Decodable, Identifiable, Sendable {
        let id: String
        let version, detail: String?
        let readyOffline: Bool
        var reference: String { version.map { "\(id)@\($0)" } ?? id }
    }
    struct StarterResult: Decodable, Sendable {
        let id, error: String?
        let template: Starter?
        let checks: [Readiness.Check]?
        let readyOffline: Bool
    }
    struct Runtime: Decodable, Identifiable, Sendable {
        let id, name, detail: String
        let version: String?
        let readyOffline: Bool
    }
    let readyOffline: Bool
    let status: String
    let docs: [Doc]
    let starters: [StarterResult]
    let runtimes: [Runtime]?
}

struct ToolboxStatus: Decodable, Sendable {
    struct Dependencies: Decodable, Sendable {
        let state: String
        let detail: String?
        var verified: Bool { state == "verified_project" }
    }
    let readyOffline: Bool
    let status: String
    let library: OfflineStatus
    let infrastructure: InfrastructureStack?
    let images: [InfrastructureStatus.Image]
    let docker: InfrastructureStatus.Docker?
    let projectDependencies: Dependencies
}
