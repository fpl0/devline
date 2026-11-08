import Foundation

struct DispatchError: LocalizedError {
    let message: String
    init(_ message: String) { self.message = message }
    var errorDescription: String? { message }
}

struct Release: Codable, Equatable {
    let id: String
    let title: String
    let notes: String

    init(title: String, notes: String) throws {
        let title = title.trimmingCharacters(in: .whitespacesAndNewlines)
        let notes = notes.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !title.isEmpty, title.utf8.count <= 100, title.rangeOfCharacter(from: .controlCharacters) == nil,
              !notes.isEmpty, notes.utf8.count <= 2_000 else {
            throw DispatchError("Enter a single-line title (1–100 bytes) and notes (1–2,000 bytes).")
        }
        self.id = UUID().uuidString.lowercased()
        self.title = title
        self.notes = notes
    }

    var email: String {
        [
            "From: releases@example.test",
            "To: team@example.test",
            "Subject: Release dispatch \(id)",
            "MIME-Version: 1.0",
            "Content-Type: text/plain; charset=UTF-8",
            "Content-Transfer-Encoding: base64",
            "",
            Data("\(title)\n\n\(notes)\n".utf8).base64EncodedString(options: [.lineLength76Characters, .endLineWithCarriageReturn, .endLineWithLineFeed]),
            "",
        ].joined(separator: "\r\n")
    }
}

enum Request {
    case publish(String, String)
    case read(String)
    case search(String)
    case help
    case selfTest

    init(_ arguments: [String]) throws {
        switch arguments.first {
        case nil:
            self = .publish("Offline release checklist", "Validate the build, announce the release and archive the checklist.")
        case "publish" where arguments.count == 3:
            self = .publish(arguments[1], arguments[2])
        case "read" where arguments.count == 2:
            guard let id = UUID(uuidString: arguments[1]) else { throw DispatchError("Use a release UUID.") }
            self = .read(id.uuidString.lowercased())
        case "search" where arguments.count == 2:
            self = .search(arguments[1])
        case "--help" where arguments.count == 1:
            self = .help
        case "--self-test" where arguments.count == 1:
            self = .selfTest
        default:
            throw DispatchError("Use publish TITLE NOTES, read ID, search QUERY, or --help.")
        }
    }
}

@main
struct ReleaseDispatch {
    static func main() async {
        do {
            let request = try Request(Array(CommandLine.arguments.dropFirst()))
            switch request {
            case let .publish(title, notes):
                let release = try Release(title: title, notes: notes)
                let services = try Services()
                try await services.publish(release)
                print("Published \(release.id) · \(release.title)")
                print("SeaweedFS archived the brief. Redis cached it. RabbitMQ handed it to Mailpit.")
                print("Meilisearch indexed the release and NATS delivered its activity event.")
            case let .read(id):
                let release = try await Services().read(id)
                print("\(release.title)\n\n\(release.notes)")
            case let .search(query):
                for release in try await Services().search(query) {
                    print("\(release.id) · \(release.title)")
                }
            case .help:
                print("With no arguments, publish the bundled sample through Devline. Commands: publish TITLE NOTES, read ID, search QUERY.")
            case .selfTest:
                try selfTest()
            }
        } catch {
            FileHandle.standardError.write(Data("\(error.localizedDescription)\n".utf8))
            exit(1)
        }
    }

    private static func selfTest() throws {
        let release = try Release(title: "  Sample release ", notes: "Notify the team.")
        let data = try JSONEncoder().encode(release)
        guard try JSONDecoder().decode(Release.self, from: data) == release,
              release.title == "Sample release",
              release.email.contains("\r\n\r\n"),
              release.email.contains(Data("Sample release\n\nNotify the team.\n".utf8).base64EncodedString()),
              (try? Release(title: "Bad\nTitle", notes: "Example")) == nil,
              (try? Request(["read", "../secret"])) == nil,
              try Request([]).isSample else {
            throw DispatchError("Release dispatch self-test failed.")
        }
        print("Release dispatch formatting and input checks passed.")
    }
}

private extension Request {
    var isSample: Bool {
        if case .publish("Offline release checklist", _) = self { return true }
        return false
    }
}
