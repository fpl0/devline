// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "ExpenseSummary",
    platforms: [.macOS(.v14)],
    products: [.executable(name: "ExpenseSummary", targets: ["ExpenseSummary"])],
    targets: [
        .target(name: "ExpenseLedger"),
        .executableTarget(name: "ExpenseSummary", dependencies: ["ExpenseLedger"]),
        .testTarget(name: "ExpenseLedgerTests", dependencies: ["ExpenseLedger"]),
    ]
)
