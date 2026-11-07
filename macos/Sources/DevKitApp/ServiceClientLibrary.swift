import Foundation

struct ServiceClientCatalog: Decodable, Sendable {
    struct Service: Decodable, Identifiable, Sendable {
        let id, name: String
        let libraries: [Library]
    }
    struct Library: Decodable, Identifiable, Sendable {
        struct Package: Decodable, Sendable {
            let name: String
            let version: String?
        }
        let id, name, detail: String
        let packages: [Package]
        let sourceUrl, support: String?
        let installed, offlineAvailable, builtin, supported: Bool
        var packageSummary: String {
            packages.map { package in package.version.map { "\(package.name) \($0)" } ?? package.name }.joined(separator: " · ")
        }
    }
    let stack: String?
    let services: [Service]
}
