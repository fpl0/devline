import Foundation

struct RecentProject: Codable, Identifiable, Equatable, Sendable {
    let path: String
    let openedAt: Date
    var id: String { path }
    var name: String { URL(fileURLWithPath: path).lastPathComponent }

    static func remembering(_ path: String, in projects: [RecentProject], now: Date = Date()) -> [RecentProject] {
        let path = URL(fileURLWithPath: path).standardizedFileURL.resolvingSymlinksInPath().path
        return Array(([RecentProject(path: path, openedAt: now)] + projects.filter { $0.path != path }).prefix(8))
    }
}
