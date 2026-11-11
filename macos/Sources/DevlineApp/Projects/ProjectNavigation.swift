import Foundation

struct RecentProject: Codable, Identifiable, Equatable, Sendable {
    static let storageKey = "devline.recentProjects"

    let path: String
    let openedAt: Date
    var id: String { path }
    var name: String { URL(fileURLWithPath: path).lastPathComponent }

    static func restore(_ data: Data?) -> [RecentProject] {
        guard let data else { return [] }
        return (try? JSONDecoder().decode([RecentProject].self, from: data)) ?? []
    }

    /// Projects in a fixed order for the sidebar: by name, so opening one never moves it.
    static func listed(_ projects: [RecentProject]) -> [RecentProject] {
        projects.sorted {
            let order = $0.name.localizedStandardCompare($1.name)
            return order == .orderedSame ? $0.path < $1.path : order == .orderedAscending
        }
    }

    static func remembering(_ path: String, in projects: [RecentProject], now: Date = Date()) -> [RecentProject] {
        let path = URL(fileURLWithPath: path).standardizedFileURL.resolvingSymlinksInPath().path
        return Array(([RecentProject(path: path, openedAt: now)] + projects.filter { $0.path != path }).prefix(8))
    }
}

/// Whether two paths name the same project folder, however they were spelled.
func sameProject(_ path: String, _ other: String?) -> Bool {
    guard let other else { return false }
    func canonical(_ value: String) -> String {
        URL(fileURLWithPath: value).standardizedFileURL.resolvingSymlinksInPath().path
    }
    return path == other || canonical(path) == canonical(other)
}

/// A short lowercase name that is valid for a new project folder.
func projectSlug(for starter: String) -> String {
    let leaf = starter.split(separator: "/").last.map(String.init) ?? starter
    let cleaned = leaf.lowercased().map { $0.isASCII && ($0.isLetter || $0.isNumber) ? $0 : "-" }
    let slug = String(cleaned).split(separator: "-").joined(separator: "-")
    guard let first = slug.first, first.isLetter else { return "project" }
    return String(slug.prefix(48)).trimmingCharacters(in: CharacterSet(charactersIn: "-"))
}
