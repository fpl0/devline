import Foundation

func displayStack(_ stack: String) -> String {
    switch stack.lowercased() {
    case "dotnet", ".net": ".NET"
    case "javascript": "JavaScript"
    case "typescript": "TypeScript"
    case "node", "nodejs": "JavaScript / TypeScript"
    default: stack.capitalized
    }
}

func shellQuote(_ text: String) -> String {
    "'" + text.replacingOccurrences(of: "'", with: "'\\''") + "'"
}

func displayDocVersion(_ version: String) -> String {
    let components = version.components(separatedBy: "+devline.")
    guard components.count == 2 else { return version }
    return "\(components[0]) · revision \(components[1])"
}

func storageSize(_ bytes: Int64) -> String {
    ByteCountFormatter.string(fromByteCount: bytes, countStyle: .file)
}
