// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "ReleaseDispatch",
    platforms: [.macOS(.v14)],
    targets: [.executableTarget(name: "ReleaseDispatch")]
)
