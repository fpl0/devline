import SwiftUI

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
    if let imported = version.range(of: "+devdocs.") { return String(version[..<imported.lowerBound]) }
    let components = version.components(separatedBy: "+devline.")
    guard components.count == 2 else { return version }
    return "\(components[0]) · revision \(components[1])"
}

func storageSize(_ bytes: Int64) -> String {
    ByteCountFormatter.string(fromByteCount: bytes, countStyle: .file)
}

func displayAppType(_ type: String) -> String {
    switch type.lowercased() {
    case "api", "http-api": "HTTP API"
    case "cli": "CLI"
    case "web", "web-app": "Web App"
    case "desktop": "Desktop App"
    case "service-kit": "Service Kit"
    default: type.replacingOccurrences(of: "-", with: " ").capitalized
    }
}

func appTypeSymbol(_ type: String) -> String {
    switch type.lowercased() {
    case "desktop": "macwindow"
    case "cli": "terminal"
    case "web", "web-app": "globe"
    case "worker": "gearshape.2"
    case "automation": "wand.and.stars"
    case "service-kit": "shippingbox"
    default: "curlybraces"
    }
}

/// A quiet identifying color for a stack, used for small glyphs only.
func stackTint(_ stack: String) -> Color {
    switch stack.lowercased() {
    case "python": .blue
    case "go": .cyan
    case "rust": .orange
    case "ruby": .red
    case "node", "javascript", "typescript": .green
    case "java": .brown
    case "dotnet": .purple
    case "elixir": .indigo
    case "swift": .pink
    default: .gray
    }
}
