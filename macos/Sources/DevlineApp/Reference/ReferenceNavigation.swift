import Foundation

struct ReferenceLocation: Equatable, Codable {
    var id: String
    var query = ""
    var stack: String?
    var pack: String?
    var category: String?
    var anchor: String?
    var focusText: String?
    var position: Double?
    var anchorOffset: Double?
    var browseIndex: Int?
}

struct ReferenceHistory: Codable {
    private(set) var entries: [ReferenceLocation] = []
    private(set) var index = -1
    private var lastSearchAt: Date?
    var current: ReferenceLocation? { entries.indices.contains(index) ? entries[index] : nil }
    var canGoBack: Bool { index > 0 }
    var canGoForward: Bool { index >= 0 && index + 1 < entries.count }

    init() {}

    private enum CodingKeys: String, CodingKey { case schemaVersion, entries, index }
    init(from decoder: Decoder) throws {
        let values = try decoder.container(keyedBy: CodingKeys.self)
        let version = try values.decode(Int.self, forKey: .schemaVersion)
        let saved = try values.decode([ReferenceLocation].self, forKey: .entries)
        let selected = try values.decode(Int.self, forKey: .index)
        guard version == 1, saved.count <= 100,
              saved.isEmpty ? selected == -1 : saved.indices.contains(selected),
              saved.allSatisfy({ !$0.id.isEmpty && $0.id.utf8.count <= 8192 && $0.query.utf8.count <= 8192
                  && ($0.position == nil || ($0.position!.isFinite && (0...1).contains($0.position!)))
                  && ($0.anchorOffset == nil || ($0.anchorOffset!.isFinite && abs($0.anchorOffset!) <= 1_000_000))
                  && ($0.browseIndex == nil || (0...100_000).contains($0.browseIndex!)) }) else {
            throw DecodingError.dataCorrupted(.init(codingPath: decoder.codingPath, debugDescription: "Invalid reading session"))
        }
        entries = saved
        index = selected
    }

    func encode(to encoder: Encoder) throws {
        var values = encoder.container(keyedBy: CodingKeys.self)
        try values.encode(1, forKey: .schemaVersion)
        try values.encode(entries, forKey: .entries)
        try values.encode(index, forKey: .index)
    }

    mutating func record(_ location: ReferenceLocation, searchPreview: Bool, now: Date = Date()) {
        if let current, current.id == location.id, current.query == location.query,
           current.stack == location.stack, current.pack == location.pack, current.category == location.category,
           current.anchor == location.anchor, current.focusText == location.focusText {
            if !searchPreview { lastSearchAt = nil }
            if let browseIndex = location.browseIndex { entries[index].browseIndex = browseIndex }
            return
        }
        if index + 1 < entries.count { entries.removeSubrange((index + 1)..<entries.count) }
        if searchPreview, let lastSearchAt, now.timeIntervalSince(lastSearchAt) < 1,
           let current, current.stack == location.stack, current.pack == location.pack, current.category == location.category {
            entries[index] = location
        } else {
            entries.append(location)
            if entries.count > 100 { entries.removeFirst(entries.count - 100) }
            index = entries.count - 1
        }
        lastSearchAt = searchPreview ? now : nil
    }

    mutating func updatePosition(id: String, fraction: Double, anchor: String? = nil, anchorOffset: Double? = nil) {
        guard current?.id == id, fraction.isFinite else { return }
        entries[index].position = min(1, max(0, fraction))
        if let anchor, let anchorOffset, anchorOffset.isFinite {
            entries[index].anchor = anchor
            entries[index].anchorOffset = anchorOffset
        }
    }

    mutating func commitSearchPreview() {
        lastSearchAt = nil
    }

    mutating func move(_ direction: Int) -> ReferenceLocation? {
        let target = index + direction
        guard entries.indices.contains(target) else { return nil }
        index = target
        lastSearchAt = nil
        return entries[index]
    }
}
