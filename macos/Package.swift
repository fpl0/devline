// swift-tools-version: 5.9
import Foundation
import PackageDescription

let coreRelease = URL(fileURLWithPath: #filePath)
    .deletingLastPathComponent()
    .deletingLastPathComponent()
    .appendingPathComponent("core/target/release").path

let package = Package(
    name: "DevKitMac",
    platforms: [.macOS(.v14)],
    products: [.executable(name: "DevKitApp", targets: ["DevKitApp"])],
    targets: [
        .systemLibrary(name: "DevKitBridge"),
        .executableTarget(
            name: "DevKitApp",
            dependencies: ["DevKitBridge"],
            linkerSettings: [
                .unsafeFlags(["-L", coreRelease]),
                .linkedLibrary("devkit_core"),
                .linkedLibrary("sqlite3"),
                .linkedFramework("Security"),
                .linkedFramework("SystemConfiguration")
            ]
        )
    ]
)
