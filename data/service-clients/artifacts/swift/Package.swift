// swift-tools-version: 6.1
import PackageDescription
let package = Package(name: "DevlineClients", platforms: [.macOS(.v14)], dependencies: [
 .package(url: "https://github.com/vapor/postgres-nio.git", exact: "1.33.3"),
 .package(url: "https://github.com/vapor/mysql-nio.git", exact: "1.9.3"),
 .package(url: "https://github.com/michaelklishin/bunny-swift.git", exact: "0.13.0"),
 .package(url: "https://github.com/swift-server/RediStack.git", exact: "1.6.3"),
 .package(url: "https://github.com/orlandos-nl/MongoKitten.git", exact: "7.16.3"),
 .package(url: "https://github.com/nats-io/nats.swift.git", exact: "0.4.0"),
 .package(url: "https://github.com/meilisearch/meilisearch-swift.git", exact: "0.17.0"),
 .package(url: "https://github.com/soto-project/soto.git", exact: "7.17.0"),
 .package(url: "https://github.com/openid/AppAuth-iOS.git", exact: "3.0.0")
], targets: [.executableTarget(name: "DevlineClients", dependencies: [
 .product(name: "PostgresNIO", package: "postgres-nio"),
 .product(name: "MySQLNIO", package: "mysql-nio"),
 .product(name: "BunnySwift", package: "bunny-swift"),
 .product(name: "RediStack", package: "RediStack"),
 .product(name: "MongoKitten", package: "MongoKitten"),
 .product(name: "Nats", package: "nats.swift"),
 .product(name: "MeiliSearch", package: "meilisearch-swift"),
 .product(name: "SotoS3", package: "soto"),
 .product(name: "AppAuth", package: "AppAuth-iOS")
])])
