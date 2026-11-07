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
    dependencies: [.package(url: "https://github.com/sparkle-project/Sparkle", exact: "2.10.1")],
    targets: [
        .systemLibrary(name: "DevKitBridge"),
        .executableTarget(
            name: "DevKitApp",
            dependencies: ["DevKitBridge", .product(name: "Sparkle", package: "Sparkle")],
            linkerSettings: [
                .unsafeFlags(["-L", coreRelease]),
                .unsafeFlags(["-Xlinker", "-rpath", "-Xlinker", "@executable_path/../Frameworks"]),
                .linkedLibrary("devkit_core"),
                .linkedLibrary("sqlite3"),
                .linkedFramework("Security"),
                .linkedFramework("SystemConfiguration")
            ]
        )
    ]
)
