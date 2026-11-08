import CryptoKit
import Foundation

struct Services {
    private let redis: URL
    private let nats: URL
    private let smtp: URL
    private let rabbit: URL
    private let rabbitPassword: String
    private let searchURL: URL
    private let searchKey: String
    private let storage: URL
    private let storageKey: String
    private let session: URLSession

    init(environment: [String: String] = ProcessInfo.processInfo.environment) throws {
        func setting(_ name: String) throws -> String {
            guard let value = environment["DEVLINE_\(name)"], !value.isEmpty else {
                throw DispatchError("Missing DEVLINE_\(name). Run from the generated Devline project.")
            }
            return value
        }
        func endpoint(_ name: String, scheme: String) throws -> URL {
            let raw = try setting(name)
            guard let url = URL(string: raw), url.scheme == scheme, url.host == "127.0.0.1",
                  let port = url.port, (1...65535).contains(port) else {
                throw DispatchError("Expected a local Devline endpoint for \(name).")
            }
            return url
        }

        redis = try endpoint("REDIS_URL", scheme: "redis")
        nats = try endpoint("NATS_URL", scheme: "nats")
        smtp = try endpoint("MAILPIT_URL", scheme: "smtp")
        let rabbitPort = try setting("RABBITMQ_MANAGEMENT_PORT")
        guard let port = Int(rabbitPort), (1...65535).contains(port),
              let rabbit = URL(string: "http://127.0.0.1:\(port)") else {
            throw DispatchError("Expected a local RabbitMQ management port.")
        }
        self.rabbit = rabbit
        rabbitPassword = try setting("RABBITMQ_PASSWORD")
        searchURL = try endpoint("MEILISEARCH_URL", scheme: "http")
        searchKey = try setting("MEILISEARCH_PASSWORD")
        storage = try endpoint("SEAWEEDFS_URL", scheme: "http")
        storageKey = try setting("SEAWEEDFS_PASSWORD")
        let configuration = URLSessionConfiguration.ephemeral
        configuration.timeoutIntervalForRequest = 5
        configuration.timeoutIntervalForResource = 10
        session = URLSession(configuration: configuration)
    }

    func publish(_ release: Release) async throws {
        let brief = try JSONEncoder().encode(release)
        _ = try await object("PUT", id: release.id, body: brief)
        try cache(release.id, brief: brief)
        try await index(release)
        let delivery = try await handoff(release.id, brief: brief)
        try mail(delivery)
        try publishActivity(release.id)
    }

    func read(_ id: String) async throws -> Release {
        if let cached = try cachedBrief(id) {
            return try JSONDecoder().decode(Release.self, from: cached)
        }
        let brief = try await object("GET", id: id)
        try cache(id, brief: brief)
        return try JSONDecoder().decode(Release.self, from: brief)
    }

    func search(_ query: String) async throws -> [Release] {
        struct Hits: Decodable { let hits: [Release] }
        var components = URLComponents(url: searchURL.appending(path: "indexes/releases/search"), resolvingAgainstBaseURL: false)!
        components.queryItems = [URLQueryItem(name: "q", value: query)]
        let data = try await http("Meilisearch", "GET", components.url!, authorization: "Bearer \(searchKey)")
        return try JSONDecoder().decode(Hits.self, from: data).hits
    }

    private func http(
        _ service: String, _ method: String, _ url: URL,
        body: Data? = nil, authorization: String? = nil, headers: [String: String] = [:]
    ) async throws -> Data {
        var request = URLRequest(url: url)
        request.httpMethod = method
        request.httpBody = body
        request.setValue(authorization, forHTTPHeaderField: "Authorization")
        if body != nil { request.setValue("application/json", forHTTPHeaderField: "Content-Type") }
        for (key, value) in headers { request.setValue(value, forHTTPHeaderField: key) }
        let (data, response) = try await session.data(for: request)
        guard let status = (response as? HTTPURLResponse)?.statusCode, (200..<300).contains(status) else {
            let status = (response as? HTTPURLResponse)?.statusCode ?? 0
            throw DispatchError("\(service) rejected \(method) \(url.path) (HTTP \(status)).")
        }
        return data
    }

    private func object(_ method: String, id: String, body: Data? = nil) async throws -> Data {
        let url = storage.appending(path: "devline/releases/\(id).json")
        let host = "127.0.0.1:\(storage.port!)"
        let time = Date.now.ISO8601Format(.iso8601.year().month().day().time(includingFractionalSeconds: false).timeZone(separator: .omitted))
            .replacingOccurrences(of: "-", with: "").replacingOccurrences(of: ":", with: "")
        let date = String(time.prefix(8))
        let scope = "\(date)/us-east-1/s3/aws4_request"
        let hash = hex(SHA256.hash(data: body ?? Data()))
        let signedHeaders = "host;x-amz-content-sha256;x-amz-date"
        let canonical = "\(method)\n\(url.path)\n\nhost:\(host)\nx-amz-content-sha256:\(hash)\nx-amz-date:\(time)\n\n\(signedHeaders)\n\(hash)"
        let toSign = "AWS4-HMAC-SHA256\n\(time)\n\(scope)\n\(hex(SHA256.hash(data: Data(canonical.utf8))))"
        let signingKey = [date, "us-east-1", "s3", "aws4_request"].reduce(Data("AWS4\(storageKey)".utf8)) {
            Data(sign($1, key: $0))
        }
        let signature = hex(sign(toSign, key: signingKey))
        let auth = "AWS4-HMAC-SHA256 Credential=devline/\(scope), SignedHeaders=\(signedHeaders), Signature=\(signature)"
        return try await http("SeaweedFS", method, url, body: body, authorization: auth, headers: [
            "x-amz-content-sha256": hash, "x-amz-date": time,
        ])
    }

    private func sign(_ message: String, key: Data) -> HMAC<SHA256>.MAC {
        HMAC<SHA256>.authenticationCode(for: Data(message.utf8), using: SymmetricKey(data: key))
    }

    private func hex(_ bytes: some Sequence<UInt8>) -> String {
        bytes.map { String(format: "%02x", $0) }.joined()
    }

    private func index(_ release: Release) async throws {
        struct Queued: Decodable { let taskUid: Int }
        struct TaskState: Decodable { let status: String }
        let endpoint = URL(string: "\(searchURL.absoluteString)/indexes/releases/documents?primaryKey=id")!
        let key = "Bearer \(searchKey)"
        let queued = try await http("Meilisearch", "POST", endpoint, body: JSONEncoder().encode([release]), authorization: key)
        let task = try JSONDecoder().decode(Queued.self, from: queued).taskUid
        for _ in 0..<40 {
            let state = try await http("Meilisearch", "GET", searchURL.appending(path: "tasks/\(task)"), authorization: key)
            switch try JSONDecoder().decode(TaskState.self, from: state).status {
            case "succeeded": return
            case "enqueued", "processing": try await Task.sleep(for: .milliseconds(250))
            default: throw DispatchError("Meilisearch could not index the release.")
            }
        }
        throw DispatchError("Meilisearch indexing timed out.")
    }

    private func handoff(_ id: String, brief: Data) async throws -> Release {
        let queue = "release-\(id)"
        let auth = "Basic \(Data("devline:\(rabbitPassword)".utf8).base64EncodedString())"
        let destination = rabbit.appending(path: "api/queues/devline/\(queue)")
        func json(_ value: [String: Any]) throws -> Data { try JSONSerialization.data(withJSONObject: value) }
        _ = try await http("RabbitMQ", "PUT", destination, body: json([
            "durable": true, "auto_delete": false, "arguments": ["x-expires": 3_600_000],
        ]), authorization: auth)
        struct Routed: Decodable { let routed: Bool }
        let routed = try await http("RabbitMQ", "POST", rabbit.appending(path: "api/exchanges/devline/amq.default/publish"), body: json([
            "routing_key": queue, "properties": ["delivery_mode": 2], "payload": brief.base64EncodedString(), "payload_encoding": "base64",
        ]), authorization: auth)
        guard try JSONDecoder().decode(Routed.self, from: routed).routed else {
            throw DispatchError("RabbitMQ did not route the release notification.")
        }
        struct Message: Decodable { let payload: String }
        let deliveries = try await http("RabbitMQ", "POST", destination.appending(path: "get"), body: json([
            "count": 1, "ackmode": "ack_requeue_false", "encoding": "base64",
        ]), authorization: auth)
        guard let delivery = try JSONDecoder().decode([Message].self, from: deliveries).first,
              let payload = Data(base64Encoded: delivery.payload), payload == brief else {
            throw DispatchError("RabbitMQ did not deliver the release notification.")
        }
        return try JSONDecoder().decode(Release.self, from: payload)
    }

    private func mail(_ release: Release) throws {
        let client = Process()
        client.executableURL = URL(fileURLWithPath: "/usr/bin/curl")
        client.arguments = [
            "--disable", "--silent", "--show-error", "--max-time", "10", "--noproxy", "*",
            "--mail-from", "releases@example.test", "--mail-rcpt", "team@example.test",
            "--upload-file", "-", smtp.absoluteString,
        ]
        let message = Pipe()
        client.standardInput = message
        try client.run()
        try message.fileHandleForWriting.write(contentsOf: Data(release.email.utf8))
        try message.fileHandleForWriting.close()
        client.waitUntilExit()
        guard client.terminationStatus == 0 else { throw DispatchError("Mailpit did not accept the release notification.") }
    }

    private func cache(_ id: String, brief: Data) throws {
        let connection = try redisConnection()
        try connection.redisCommand([Data("SETEX".utf8), Data("release:\(id)".utf8), Data("600".utf8), brief])
        guard try connection.line() == "+OK" else { throw DispatchError("Redis rejected the release brief.") }
    }

    private func cachedBrief(_ id: String) throws -> Data? {
        let connection = try redisConnection()
        try connection.redisCommand([Data("GET".utf8), Data("release:\(id)".utf8)])
        let response = try connection.line()
        if response == "$-1" { return nil }
        guard response.first == "$", let size = Int(response.dropFirst()), (0...16_384).contains(size) else {
            throw DispatchError("Redis returned an invalid release brief.")
        }
        return try connection.read(size)
    }

    private func redisConnection() throws -> LoopbackConnection {
        let connection = try LoopbackConnection("Redis", port: redis.port!)
        try connection.redisCommand([Data("AUTH".utf8), Data((redis.password ?? "").utf8)])
        guard try connection.line() == "+OK" else { throw DispatchError("Redis rejected the Devline credentials.") }
        return connection
    }

    private func publishActivity(_ id: String) throws {
        let connection = try LoopbackConnection("NATS", port: nats.port!)
        guard try connection.line().hasPrefix("INFO ") else { throw DispatchError("NATS did not send its greeting.") }
        let credentials = try JSONSerialization.data(withJSONObject: [
            "user": nats.user ?? "", "pass": nats.password ?? "", "verbose": false,
        ])
        let event = try JSONSerialization.data(withJSONObject: ["releaseId": id, "event": "published"])
        var command = Data("CONNECT ".utf8)
        command.append(credentials)
        command.append(Data("\r\nSUB releases.published 1\r\nPUB releases.published \(event.count)\r\n".utf8))
        command.append(event)
        command.append(Data("\r\nPING\r\n".utf8))
        try connection.send(command)
        let frame = try connection.line()
        guard frame == "MSG releases.published 1 \(event.count)",
              try connection.read(event.count) == event else {
            throw DispatchError("NATS did not deliver the release event.")
        }
    }
}
