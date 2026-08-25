// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "VocabMacBridge",
    platforms: [.macOS(.v15)],
    products: [.library(name: "VocabMacBridge", type: .static, targets: ["VocabMacBridge"])],
    targets: [
        .target(name: "VocabMacBridge"),
        .testTarget(name: "VocabMacBridgeTests", dependencies: ["VocabMacBridge"]),
    ]
)
