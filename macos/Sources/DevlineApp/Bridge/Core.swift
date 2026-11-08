import Darwin
import DevlineBridge
import Foundation

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
    var clients: [String]?
    var tail: Int?
    var ports: [String: Int]?
    var offline: Bool?
    var trustImages: Bool?
    var manifestSha256: String?
    var keepLatest: Bool?
    var confirmedOnly: Bool?

    init(
        _ command: String, id: String? = nil, version: String? = nil,
        query: String? = nil, stack: String? = nil, packId: String? = nil, limit: Int? = nil, offset: Int? = nil,
        destination: String? = nil, path: String? = nil, dryRun: Bool? = nil, url: String? = nil,
        anchor: String? = nil, focusText: String? = nil, category: String? = nil, projectStack: String? = nil,
        projectPath: String? = nil, kind: String? = nil, docs: [String]? = nil, starters: [String]? = nil, trustStarters: Bool? = nil,
        jobId: String? = nil, images: [String]? = nil, services: [String]? = nil, clients: [String]? = nil, tail: Int? = nil, ports: [String: Int]? = nil, offline: Bool? = nil, trustImages: Bool? = nil, manifestSha256: String? = nil, keepLatest: Bool? = nil, confirmedOnly: Bool? = nil
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
        self.clients = clients
        self.tail = tail
        self.ports = ports
        self.offline = offline
        self.trustImages = trustImages
        self.manifestSha256 = manifestSha256
        self.keepLatest = keepLatest
        self.confirmedOnly = confirmedOnly
    }

    private static var catalogPath: String {
        let environment = ProcessInfo.processInfo.environment
        for name in ["DEVLINE_CATALOG", "DEVKIT_CATALOG"] {
            if let override = environment[name], !override.isEmpty { return override }
        }
        if let bundled = Bundle.main.resourceURL?.appendingPathComponent("catalog"),
           FileManager.default.fileExists(atPath: bundled.appendingPathComponent("catalog.json").path) {
            return bundled.path
        }
        return URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent().deletingLastPathComponent()
            .deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
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
            guard let pointer = json.withCString({ devline_request($0) }) else {
                throw CoreError(message: "Devline could not read the local library.")
            }
            defer { devline_free(pointer) }
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
