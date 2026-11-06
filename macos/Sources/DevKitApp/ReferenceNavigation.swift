import Foundation

struct ReferenceLocation: Equatable {
    var id: String
    var query = ""
    var stack: String?
    var pack: String?
    var category: String?
    var anchor: String?
    var focusText: String?
    var position: Double?
    var browseIndex: Int?
}

struct ReferenceHistory {
    private(set) var entries: [ReferenceLocation] = []
    private(set) var index = -1
    private var lastSearchAt: Date?
    var current: ReferenceLocation? { entries.indices.contains(index) ? entries[index] : nil }
    var canGoBack: Bool { index > 0 }
    var canGoForward: Bool { index >= 0 && index + 1 < entries.count }

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

    mutating func updatePosition(id: String, fraction: Double) {
        guard current?.id == id, fraction.isFinite else { return }
        entries[index].position = min(1, max(0, fraction))
    }

    mutating func move(_ direction: Int) -> ReferenceLocation? {
        let target = index + direction
        guard entries.indices.contains(target) else { return nil }
        index = target
        lastSearchAt = nil
        return entries[index]
    }
}
