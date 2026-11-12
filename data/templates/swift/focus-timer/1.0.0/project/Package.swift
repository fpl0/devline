// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "FocusSessions",
    platforms: [.macOS(.v14)],
    products: [.executable(name: "FocusSessions", targets: ["FocusSessions"])],
    targets: [
        .target(name: "FocusCore"),
        .executableTarget(name: "FocusSessions", dependencies: ["FocusCore"]),
        .testTarget(name: "FocusCoreTests", dependencies: ["FocusCore"]),
    ]
)
