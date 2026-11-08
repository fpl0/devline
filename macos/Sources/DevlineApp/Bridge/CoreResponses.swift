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

struct Starter: Decodable, Hashable, Identifiable, Sendable {
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
    let matchAnchor: String?
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

struct LibraryRetention: Decodable, Sendable {
    let keepLatest: Bool
}

struct LibraryPurgeResult: Decodable, Sendable {
    let docs, starters, images: Int

    var summary: String {
        if docs + starters + images == 0 { return "No older versions to purge." }
        return "Purged older releases (\(docs) docsets, \(starters) starters, \(images) image archives)."
    }
}
