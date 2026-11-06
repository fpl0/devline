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

    var reference: String { "\(id)@\(version)" }
}

struct Starter: Decodable, Hashable, Sendable {
    let id, name, stack, version, description, appType, toolchain: String
    let minimumVersion, testCommand, target: String
    let docs: [String]
    let verified: Bool
    let installed: Bool?

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
    let checks: [Check]
}

struct Project: Decodable, Sendable {
    let path, version, testCommand: String
}

struct ProjectContext: Decodable, Sendable {
    let path: String
    let stacks: [String]
    let stack: String?
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

    init(
        _ command: String, id: String? = nil, version: String? = nil,
        query: String? = nil, stack: String? = nil, packId: String? = nil, limit: Int? = nil, offset: Int? = nil,
        destination: String? = nil, path: String? = nil, dryRun: Bool? = nil, url: String? = nil,
        anchor: String? = nil, focusText: String? = nil, category: String? = nil, projectStack: String? = nil
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
    case "nodejs": "Node.js"
    default: stack.capitalized
    }
}

func shellQuote(_ text: String) -> String {
    "'" + text.replacingOccurrences(of: "'", with: "'\\''") + "'"
}
