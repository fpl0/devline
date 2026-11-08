import Foundation

@main struct ReferenceNavigationTests {
    static func main() {
        var history = ReferenceHistory()
        let first = ReferenceLocation(id: "python@3/a", query: "Path", stack: "python")
        history.record(first, searchPreview: false)
        history.updatePosition(id: first.id, fraction: 0.6, anchor: "section", anchorOffset: -132)
        history.record(ReferenceLocation(id: "python@3/b", query: "Path", stack: "python", anchor: "method"), searchPreview: false)
        precondition(history.canGoBack && !history.canGoForward)
        precondition(history.move(-1)?.position == 0.6)
        precondition(history.current?.anchor == "section" && history.current?.anchorOffset == -132)
        precondition(history.move(1)?.anchor == "method")
        _ = history.move(-1)
        history.record(ReferenceLocation(id: "go@1/c"), searchPreview: false)
        precondition(history.entries.count == 2 && !history.canGoForward)
        history.updatePosition(id: "stale", fraction: 0.2)
        precondition(history.current?.position == nil)
        history.updatePosition(id: "go@1/c", fraction: .infinity)
        precondition(history.current?.position == nil)
        var search = ReferenceHistory()
        let start = Date(timeIntervalSince1970: 100)
        search.record(first, searchPreview: true, now: start)
        search.record(ReferenceLocation(id: "python@3/d", query: "Path.read", stack: "python"), searchPreview: true, now: start.addingTimeInterval(0.3))
        precondition(search.entries.count == 1)
        search.record(ReferenceLocation(id: "go@1/e", query: "ReadFile", stack: "go"), searchPreview: true, now: start.addingTimeInterval(0.6))
        precondition(search.entries.count == 2)
        var confirmedSearch = ReferenceHistory()
        confirmedSearch.record(first, searchPreview: true, now: start)
        confirmedSearch.updatePosition(id: first.id, fraction: 0.4, anchor: "method", anchorOffset: 20)
        confirmedSearch.commitSearchPreview()
        precondition(confirmedSearch.entries.count == 1)
        confirmedSearch.record(ReferenceLocation(id: "python@3/d", query: "Path.read", stack: "python"), searchPreview: true, now: start.addingTimeInterval(0.4))
        precondition(confirmedSearch.entries.count == 2 && confirmedSearch.move(-1)?.position == 0.4
            && confirmedSearch.current?.anchor == "method")
        var browsing = ReferenceHistory()
        let later = ReferenceLocation(id: "python@3/collections", stack: "python", pack: "python@3", category: "api", browseIndex: 160)
        browsing.record(later, searchPreview: false)
        browsing.record(first, searchPreview: false)
        precondition(browsing.move(-1)?.browseIndex == 160)
        for n in 0..<120 { search.record(ReferenceLocation(id: "record-\(n)"), searchPreview: false) }
        precondition(search.entries.count == 100 && search.current?.id == "record-119")
        let encoded = try! JSONEncoder().encode(history)
        let restored = try! JSONDecoder().decode(ReferenceHistory.self, from: encoded)
        precondition(restored.entries == history.entries && restored.index == history.index)
        precondition(restored.canGoBack == history.canGoBack && restored.canGoForward == history.canGoForward)
        let browseRestored = try! JSONDecoder().decode(ReferenceHistory.self, from: JSONEncoder().encode(browsing))
        precondition(browseRestored.current?.browseIndex == 160)
        for invalid in [#"{"schemaVersion":9,"entries":[],"index":-1}"#,
                        #"{"schemaVersion":1,"entries":[],"index":0}"#,
                        #"{"schemaVersion":1,"entries":[{"id":"a","query":"","position":5}],"index":0}"#] {
            precondition((try? JSONDecoder().decode(ReferenceHistory.self, from: Data(invalid.utf8))) == nil)
        }
        print("Reference navigation: selection, context, position, branching, stale updates, search coalescing and bounds passed")
    }
}
