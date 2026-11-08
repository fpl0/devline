// swift-tools-version: 5.9
import Foundation
import PackageDescription

let coreRelease = URL(fileURLWithPath: #filePath)
    .deletingLastPathComponent()
    .deletingLastPathComponent()
    .appendingPathComponent("core/target/release").path

let package = Package(
    name: "DevlineMac",
    platforms: [.macOS(.v14)],
    products: [.executable(name: "DevlineApp", targets: ["DevlineApp"])],
    dependencies: [.package(url: "https://github.com/sparkle-project/Sparkle", exact: "2.10.1")],
    targets: [
        .systemLibrary(name: "DevlineBridge"),
        .executableTarget(
            name: "DevlineApp",
            dependencies: ["DevlineBridge", .product(name: "Sparkle", package: "Sparkle")],
            resources: [
                .copy("Resources/highlight.min.js"),
                .copy("Resources/code-highlighting.js"),
                .copy("Resources/MenuBarMonogram.png")
            ],
            linkerSettings: [
                .unsafeFlags(["-L", coreRelease]),
                .unsafeFlags(["-Xlinker", "-rpath", "-Xlinker", "@executable_path/../Frameworks"]),
                .linkedLibrary("devline_core"),
                .linkedLibrary("sqlite3"),
                .linkedFramework("Security"),
                .linkedFramework("SystemConfiguration")
            ]
        )
    ]
)
