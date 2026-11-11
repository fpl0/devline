import Foundation

struct Catalog: Decodable, Equatable, Sendable {
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
    let entryId: String?
    let localOnly: Bool

    var reference: String { "\(id)@\(version)" }

    private enum CodingKeys: String, CodingKey {
        case id, name, stack, version, description, sourceUrl, installed, upstreamVersion, snapshotDate, coverage, omissions, active, localOnly, entryId
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
        entryId = try values.decodeIfPresent(String.self, forKey: .entryId)
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

struct RuntimeToolchain: Decodable, Equatable, Identifiable, Sendable {
    /// How to install a runtime this Mac does not have.
    struct InstallGuide: Decodable, Equatable, Sendable {
        struct Command: Decodable, Equatable, Sendable, Identifiable {
            let manager, command: String
            let present: Bool
            var id: String { manager }
        }
        let commands: [Command]
        let url, note: String?
    }
    let id, name, toolchain, minimumVersion, detail: String
    let available: Bool
    let path: String?
    let docs, templates: [String]
    let install: InstallGuide?
}

/// One runtime's shelf of packages that can be added to projects without a connection.
struct ShelfStack: Decodable, Equatable, Identifiable, Sendable {
    struct Shelf: Decodable, Equatable, Sendable {
        struct Failure: Decodable, Equatable, Sendable { let name: String; let error: String? }
        let ready: Bool
        let count, total: Int
        let bytes: Int64?
        let preparedAt: Double?
        let failed: [Failure]?
    }
    struct Package: Decodable, Equatable, Identifiable, Sendable {
        let name: String
        let summary, category, version, error: String?
        let available: Bool
        let installed: Bool?
        var id: String { name }
    }
    let stack, registry: String
    let shelf: Shelf
    let packages: [Package]
    var id: String { stack }
}

struct ShelfCatalog: Decodable, Sendable {
    let stacks: [ShelfStack]
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
