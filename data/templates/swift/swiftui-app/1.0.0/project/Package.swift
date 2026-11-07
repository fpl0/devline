// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "IdeaBoard",
    platforms: [.macOS(.v14)],
    products: [.executable(name: "IdeaBoard", targets: ["IdeaBoard"])],
    targets: [
        .target(name: "IdeaBoardCore"),
        .executableTarget(name: "IdeaBoard", dependencies: ["IdeaBoardCore"]),
        .testTarget(name: "IdeaBoardCoreTests", dependencies: ["IdeaBoardCore"]),
    ]
)
