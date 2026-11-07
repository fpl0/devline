import Darwin
import DevKitBridge
import Foundation

struct Catalog: Decodable, Sendable {
    let docs: [DocPack]
    let templates: [Starter]
    let home: String
}

struct DocPack: Decodable, Hashable, Sendable {
    let id, name, stack, version, description: String
    let sourceUrl: String
    let installed: Bool
    let upstreamVersion, snapshotDate: String?
    let coverage, omissions: [String]?
    let active: Bool?
    let localOnly: Bool

    var reference: String { "\(id)@\(version)" }

    private enum CodingKeys: String, CodingKey {
        case id, name, stack, version, description, sourceUrl, installed, upstreamVersion, snapshotDate, coverage, omissions, active, localOnly
    }
    init(from decoder: Decoder) throws {
        let values = try decoder.container(keyedBy: CodingKeys.self)
        id = try values.decode(String.self, forKey: .id)
        name = try values.decode(String.self, forKey: .name)
        stack = try values.decode(String.self, forKey: .stack)
        version = try values.decode(String.self, forKey: .version)
        description = try values.decode(String.self, forKey: .description)
        sourceUrl = try values.decode(String.self, forKey: .sourceUrl)
        installed = try values.decodeIfPresent(Bool.self, forKey: .installed) ?? false
        upstreamVersion = try values.decodeIfPresent(String.self, forKey: .upstreamVersion)
        snapshotDate = try values.decodeIfPresent(String.self, forKey: .snapshotDate)
        coverage = try values.decodeIfPresent([String].self, forKey: .coverage)
        omissions = try values.decodeIfPresent([String].self, forKey: .omissions)
        active = try values.decodeIfPresent(Bool.self, forKey: .active)
        localOnly = try values.decodeIfPresent(Bool.self, forKey: .localOnly) ?? false
    }
}

struct LocalDocumentationStatus: Decodable, Sendable {
    let id: String
    let available, installed: Bool
    let version, sdkVersion, sdkBuild, toolchainVersion, detail: String?
    let count: Int?
}

struct Starter: Decodable, Hashable, Sendable {
    let id, name, stack, version, description, appType, toolchain: String
    let minimumVersion, testCommand, target: String
    let docs: [String]
    let verified: Bool
    let installed: Bool?
    let services: [String]?
    let minimumMacos: String?

    var reference: String { "\(id)@\(version)" }
    var isInstalled: Bool { installed == true }
}

struct SearchHit: Decodable, Identifiable, Sendable {
    let id, packId, title, stack, version, sourceUrl, snippet: String
    let matchText: String?
    let symbol: String?
    let matchKind: String?
    let category: String?
}

struct Document: Decodable, Identifiable, Sendable {
    let id, packId, title, stack, version, sourceUrl, body: String
    let html: String?
    let focusText: String?
    let previousId, nextId: String?
    let part, parts: Int?
}

struct Readiness: Decodable, Sendable {
    struct Check: Decodable, Sendable {
        let name, detail: String
        let ok: Bool
    }
    let ready: Bool
    let readyOffline: Bool?
    let checks: [Check]
}

struct Project: Decodable, Sendable {
    let path, version, testCommand: String
}

struct ProjectContext: Decodable, Sendable {
    struct Origin: Decodable, Sendable { let starter, version, reference, digest: String }
    let path: String
    let stacks: [String]
    let stack: String?
    let dependencies: [ProjectDependency]?
    let pins: [ProjectPin]?
    let docsets: [ProjectDocset]?
    let origin: Origin?
}

struct ProjectDependency: Decodable, Identifiable, Sendable {
    let id: String
    let version, requirement, source: String?
}

struct ProjectPin: Decodable, Sendable {
    let id, version: String
}

struct ProjectDocset: Decodable, Identifiable, Sendable {
    let id, state, detail: String
    let installedVersions, availableVersions: [String]
    let version, requirement, prepareVersion: String?
    let compatible: Bool?
    var needsAttention: Bool { ["unmatched", "uninstalled", "unversioned"].contains(state) || compatible == false
        || (state == "pinned" && (version == nil || !installedVersions.contains(version!))) }
}

struct RuntimeToolchain: Decodable, Identifiable, Sendable {
    let id, name, toolchain, minimumVersion, detail: String
    let available: Bool
    let path: String?
    let docs, templates: [String]
}

struct Acknowledgment: Decodable, Sendable {
    init(from decoder: Decoder) throws {}
}

struct CoreRequest: Encodable, Sendable {
    let command: String
    let catalog: String
    var id: String?
    var version: String?
    var query: String?
    var stack: String?
    var packId: String?
    var limit: Int?
    var offset: Int?
    var destination: String?
    var path: String?
    var url: String?
    var dryRun: Bool?
    var anchor: String?
    var focusText: String?
    var category: String?
    var projectStack: String?
    var projectPath: String?
    var kind: String?
    var docs: [String]?
    var starters: [String]?
    var trustStarters: Bool?
    var jobId: String?
    var images: [String]?
    var services: [String]?
    var tail: Int?
    var ports: [String: Int]?
    var offline: Bool?
    var trustImages: Bool?
    var manifestSha256: String?

    init(
        _ command: String, id: String? = nil, version: String? = nil,
        query: String? = nil, stack: String? = nil, packId: String? = nil, limit: Int? = nil, offset: Int? = nil,
        destination: String? = nil, path: String? = nil, dryRun: Bool? = nil, url: String? = nil,
        anchor: String? = nil, focusText: String? = nil, category: String? = nil, projectStack: String? = nil,
        projectPath: String? = nil, kind: String? = nil, docs: [String]? = nil, starters: [String]? = nil, trustStarters: Bool? = nil,
        jobId: String? = nil, images: [String]? = nil, services: [String]? = nil, tail: Int? = nil, ports: [String: Int]? = nil, offline: Bool? = nil, trustImages: Bool? = nil, manifestSha256: String? = nil
    ) {
        self.command = command
        self.catalog = Self.catalogPath
        self.id = id
        self.version = version
        self.query = query
        self.stack = stack
        self.packId = packId
        self.limit = limit
        self.offset = offset
        self.destination = destination
        self.path = path
        self.url = url
        self.anchor = anchor
        self.focusText = focusText
        self.category = category
        self.projectStack = projectStack
        self.dryRun = dryRun
        self.projectPath = projectPath
        self.kind = kind
        self.docs = docs
        self.starters = starters
        self.trustStarters = trustStarters
        self.jobId = jobId
        self.images = images
        self.services = services
        self.tail = tail
        self.ports = ports
        self.offline = offline
        self.trustImages = trustImages
        self.manifestSha256 = manifestSha256
    }

    private static var catalogPath: String {
        if let override = ProcessInfo.processInfo.environment["DEVKIT_CATALOG"], !override.isEmpty {
            return override
        }
        if let bundled = Bundle.main.resourceURL?.appendingPathComponent("catalog"),
           FileManager.default.fileExists(atPath: bundled.appendingPathComponent("catalog.json").path) {
            return bundled.path
        }
        return URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent().deletingLastPathComponent()
            .deletingLastPathComponent().deletingLastPathComponent()
            .appendingPathComponent("data").path
    }
}

private struct Envelope<Result: Decodable>: Decodable {
    let ok: Bool
    let data: Result?
    let error: String?
    private enum CodingKeys: String, CodingKey { case ok, data, error }
    init(from decoder: Decoder) throws {
        let values = try decoder.container(keyedBy: CodingKeys.self)
        ok = try values.decode(Bool.self, forKey: .ok)
        error = try values.decodeIfPresent(String.self, forKey: .error)
        data = ok ? try values.decode(Result.self, forKey: .data) : nil
    }
}

struct CoreError: LocalizedError, Sendable {
    let message: String
    var errorDescription: String? { message }
}

enum Core {
    static func request<Result: Decodable & Sendable>(_ request: CoreRequest) async throws -> Result {
        try Task.checkCancellation()
        let result = try await Task.detached(priority: .userInitiated) {
            let encoder = JSONEncoder()
            encoder.keyEncodingStrategy = .convertToSnakeCase
            let json = String(decoding: try encoder.encode(request), as: UTF8.self)
            guard let pointer = json.withCString({ devkit_request($0) }) else {
                throw CoreError(message: "Devline could not read the local library.")
            }
            defer { devkit_free(pointer) }
            let decoder = JSONDecoder()
            decoder.keyDecodingStrategy = .convertFromSnakeCase
            let data = Data(bytes: pointer, count: strlen(pointer))
            let response = try decoder.decode(Envelope<Result>.self, from: data)
            if response.ok, let result = response.data { return result }
            throw CoreError(message: response.error ?? "Devline could not complete the request.")
        }.value
        try Task.checkCancellation()
        return result
    }
}

func displayStack(_ stack: String) -> String {
    switch stack.lowercased() {
    case "dotnet", ".net": ".NET"
    case "javascript": "JavaScript"
    case "typescript": "TypeScript"
    case "node", "nodejs": "JavaScript / TypeScript"
    default: stack.capitalized
    }
}

func shellQuote(_ text: String) -> String {
    "'" + text.replacingOccurrences(of: "'", with: "'\\''") + "'"
}
