import Foundation
import SwiftUI

struct InfrastructureCatalog: Decodable, Sendable {
    let services: [InfrastructureService]
    let platform: String
}

struct InfrastructureService: Decodable, Identifiable, Sendable {
    struct Image: Decodable, Sendable {
        let registry, repository, tag, digest, configDigest, platform: String
        let compressedBytes: Int64?
    }
    struct Provenance: Decodable, Sendable {
        let publisher, imageUrl, sourceUrl, license: String
        let docsUrl: String?
    }
    struct Port: Decodable, Identifiable, Sendable {
        let id, name: String
        let containerPort, hostPort: Int
    }
    let id, name, version, category, description: String
    let image: Image
    let ports: [Port]
    let provenance: Provenance
    let docsRef, helpRef, quickStart: String?
    let docsCoverage, docsOmissions: String?
    var reference: String { "\(id)@\(version)" }
    var symbol: String {
        switch id {
        case "postgres", "postgresql", "mysql", "mongodb", "mongo": "externaldrive"
        case "redis": "bolt.horizontal.circle"
        case "rabbitmq", "nats": "arrow.triangle.branch"
        case "mailpit": "envelope"
        case "seaweedfs": "externaldrive.badge.icloud"
        case "keycloak": "key"
        case "meilisearch": "magnifyingglass"
        default: "server.rack"
        }
    }
}

struct InfrastructureStatus: Decodable, Sendable {
    struct Docker: Decodable, Sendable {
        let available: Bool
        let endpoint, platform, detail: String?
    }
    struct Receipt: Decodable, Sendable {
        let source, sourceDigest, imageId, archiveSha256: String
        let sizeBytes: Int64
        let verifiedAt: Double
    }
    struct Image: Decodable, Identifiable, Sendable {
        let id, version, platform, status: String
        let archived, readyOffline: Bool
        let loaded: Bool?
        let integrity: String?
        let receipt: Receipt?
        var reference: String { "\(id)@\(version)" }
        var verifiedArchive: Bool { archived && integrity == "verified" }
    }
    let docker: Docker
    let images: [Image]
}

struct InfrastructureStack: Decodable, Sendable {
    struct Service: Decodable, Identifiable, Sendable {
        let id, version: String
        let ports: [InfrastructureService.Port]
        var reference: String { "\(id)@\(version)" }
    }
    let path: String
    let projectName: String?
    let configured: Bool
    let services: [Service]
    let running: Bool?
    let healthy: Bool?
    let statusError: String?
    let log: String?
}

@MainActor
final class InfrastructureLibrary: ObservableObject {
    @Published private(set) var catalog: InfrastructureCatalog?
    @Published private(set) var status: InfrastructureStatus?
    @Published private(set) var loading = false
    @Published private(set) var error: String?
    private var refreshTask: Task<Void, Never>?
    private var refreshPending = false

    var archived: [InfrastructureStatus.Image] { status?.images.filter(\.archived) ?? [] }
    var sizeBytes: Int64 { archived.reduce(0) { $0 + ($1.receipt?.sizeBytes ?? 0) } }
    func image(_ reference: String) -> InfrastructureStatus.Image? { status?.images.first { $0.reference == reference } }

    func refresh() async {
        refreshPending = true
        if let refreshTask { return await refreshTask.value }
        loading = true
        let task = Task {
            defer { loading = false; refreshTask = nil }
            repeat {
                refreshPending = false
                await loadSnapshot()
            } while refreshPending
        }
        refreshTask = task
        await task.value
    }

    private func loadSnapshot() async {
        do {
            async let nextCatalog: InfrastructureCatalog = Core.request(CoreRequest("infra.catalog"))
            async let nextStatus: InfrastructureStatus = Core.request(CoreRequest("infra.status"))
            let (catalog, status) = try await (nextCatalog, nextStatus)
            self.catalog = catalog
            self.status = status
            error = nil
        } catch is CancellationError {}
        catch { self.error = error.localizedDescription }
    }
}
