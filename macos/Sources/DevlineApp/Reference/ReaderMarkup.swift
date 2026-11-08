import AppKit

struct ReaderArticle {
    let contents: NSAttributedString
    let sections: [ReaderSection]
}

enum ReaderMarkup {
    private static let markdownHeading = try! NSRegularExpression(pattern: #"^ {0,3}(#{1,6})\s+(.+?)\s*#*\s*$"#)
    private static let htmlHeading = try! NSRegularExpression(pattern: #"(?i)^\s*<h([1-6])(?:\s+[^>]*)?>(.*?)</h[1-6]>\s*$"#)
    private static let blockTag = try! NSRegularExpression(pattern: #"(?i)^\s*</?(?:details|summary|div|figure|picture|table|tbody|thead|tr|td|th|ul|ol|li|p)(?:\s+[^>]*)?\s*>\s*$"#)
    private static let imageTag = try! NSRegularExpression(pattern: #"(?i)<(?:img|source)\b(?:\s+[\w:-]+(?:\s*=\s*(?:"[^"]*"|'[^']*'|[^\s>]+))?)*\s*/?>"#)
    private static let imageMarkup = try! NSRegularExpression(pattern: #"!\[([^\]]*)\]\([^\n)]*\)"#)
    private static let proseMarkup = try! NSRegularExpression(pattern: #"`([^`\n]+)`|\*\*([^*\n]+)\*\*|\[([^\]\n]+)\]\(([^\s)]+)\)|\{([^}\n]+)\}\[([^\s\]]+)\]|<(code|em|strong|b|i|tt|span)>([^<>]*)</\7>"#, options: [.caseInsensitive])
    private static let inlineTags = try! NSRegularExpression(pattern: #"(?i)</?(?:span|sub|sup|kbd)\s+[\w:-]+\s*=\s*(?:"[^"]*"|'[^']*')[^>]*>|<br\s*/>"#)
    private static let tagOnly = try! NSRegularExpression(pattern: #"(?i)^\s*</?(?:a|div|p|picture|figure|span|img|source|details|summary)(?:\s+[^>]*)?\s*/?>\s*$"#)
    private static let goDeclaration = try! NSRegularExpression(pattern: #"^(?:type|func|var|const)\s+\S+"#)
    private static let listItem = try! NSRegularExpression(pattern: #"^(?:[-+*•]|[0-9]+[.)])\s+"#)
    private static let plainHeadings: Set<String> = [
        "overview", "introduction", "examples", "example", "usage", "basic usage", "syntax",
        "parameters", "arguments", "return value", "returns", "notes", "see also", "exceptions",
        "constructors", "constructor summary", "field summary", "method summary", "method details",
        "constructor details", "classes", "functions", "methods", "types", "constants", "variables",
        "statics", "trait implementations", "implementations", "associated types", "required methods",
        "provided methods", "auto trait implementations", "blanket implementations", "module",
        "structs", "enums", "traits", "type aliases", "macros", "callbacks", "options", "errors",
        "panics", "safety", "configuration", "details", "getting started", "references", "remarks"
    ]

    static func title(_ value: String, fallback: String) -> String {
        let text = summary(value)
        if text.isEmpty || tagOnly.firstMatch(in: text, range: NSRange(text.startIndex..., in: text)) != nil {
            return fallback
        }
        return text
    }

    static func summary(_ value: String) -> String {
        let text = cleanProse(value)
            .replacingOccurrences(of: #"^\s*#{1,6}\s+"#, with: "", options: .regularExpression)
            .replacingOccurrences(of: #"[\n\r\t]+"#, with: " ", options: .regularExpression)
        let source = text as NSString
        var output = ""
        var position = 0
        for match in proseMarkup.matches(in: text, range: NSRange(location: 0, length: source.length)) {
            output += cleanedInline(source.substring(with: NSRange(
                location: position, length: match.range.location - position
            )))
            let token = [1, 2, 3, 5, 8].compactMap { capture(match, group: $0, in: text) }.first ?? ""
            output += capture(match, group: 1, in: text) == nil ? cleanedInline(token) : token
            position = NSMaxRange(match.range)
        }
        output += cleanedInline(source.substring(from: position))
        return output.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    static func article(_ document: Document, fallback: String) -> ReaderArticle {
        let contents = NSMutableAttributedString()
        let articleTitle = title(document.title, fallback: fallback)
        let titleStyle = paragraph(spacing: 17)
        contents.append(NSAttributedString(string: articleTitle + "\n", attributes: [
            .font: NSFont.systemFont(ofSize: 29, weight: .semibold),
            .foregroundColor: NSColor.labelColor,
            .paragraphStyle: titleStyle
        ]))

        let lines = document.body.components(separatedBy: .newlines)
        var sections: [ReaderSection] = []
        var index = 0
        var codeFence: String?
        var previousBlank = true
        let localID = document.id.split(separator: "/", maxSplits: 1).last ?? ""
        let isGoDoc = (document.packId == "go" && localID.hasPrefix("std/"))
            || (document.packId == "chi" && (localID == "api" || localID.hasPrefix("api/")))
        let isGoSource = document.packId == "chi" && localID.hasPrefix("_examples/")
        // In go doc output, package examples use four spaces. After the first
        // declaration, doc comments use that indent and their examples use eight.
        var inDeclarations = isGoDoc && !(lines.first?.hasPrefix("package ") ?? false)
        var wrappedProse: [String] = []
        func flushProse() {
            guard !wrappedProse.isEmpty else { return }
            appendInline(wrappedProse.joined(separator: " "), to: contents)
            wrappedProse.removeAll(keepingCapacity: true)
        }

        while index < lines.count {
            let line = lines[index]
            let trimmed = line.trimmingCharacters(in: .whitespaces)
            let lineRange = NSRange(line.startIndex..., in: line)
            let next = lines.indices.contains(index + 1) ? lines[index + 1] : ""
            index += 1

            if let fence = codeFence {
                if trimmed.hasPrefix(fence) && trimmed.allSatisfy({ $0 == fence.first }) {
                    codeFence = nil
                } else {
                    appendCode(line, to: contents)
                }
                previousBlank = false
                continue
            }

            if !isGoSource, let marker = trimmed.first, marker == "`" || marker == "~" {
                let prefix = trimmed.prefix(while: { $0 == marker })
                if prefix.count >= 3 {
                    flushProse()
                    codeFence = String(prefix)
                    continue
                }
            }

            if trimmed.isEmpty {
                flushProse()
                if !previousBlank {
                    contents.append(NSAttributedString(string: "\n", attributes: [
                        .font: NSFont.systemFont(ofSize: 6),
                        .paragraphStyle: paragraph()
                    ]))
                }
                previousBlank = true
                continue
            }

            let declaration = (isGoDoc || isGoSource) && goDeclaration.firstMatch(in: line, range: lineRange) != nil
            if declaration { inDeclarations = true }
            let codeIndent = isGoDoc && inDeclarations ? "        " : "    "
            let isCode = isGoSource || line.hasPrefix("\t") || line.hasPrefix(codeIndent) || declaration
                || (isGoDoc && (line.hasPrefix("    \t") || line.hasPrefix("package ") || trimmed == "}" || trimmed == ")"))
            if isCode {
                flushProse()
                let start = contents.length
                appendCode(line, to: contents)
                if declaration && sections.count < 100 {
                    sections.append(ReaderSection(title: trimmed, target: .text(NSRange(location: start, length: line.utf16.count))))
                }
                previousBlank = false
                continue
            }

            if blockTag.firstMatch(in: line, range: lineRange) != nil {
                flushProse()
                continue
            }
            let nextTrimmed = next.trimmingCharacters(in: .whitespaces)
            let setext = nextTrimmed.count >= 3 && trimmed.count < 100
                && (nextTrimmed.allSatisfy { $0 == "=" } || nextTrimmed.allSatisfy { $0 == "-" })
            let heading = markdownHeading.firstMatch(in: line, range: lineRange)
                ?? htmlHeading.firstMatch(in: line, range: lineRange)
            let plainKey = trimmed.trimmingCharacters(in: CharacterSet(charactersIn: ":")).lowercased()
            let plainHeading = previousBlank && nextTrimmed.isEmpty && plainHeadings.contains(plainKey)
            if let heading {
                flushProse()
                let label = capture(heading, group: 2, in: line) ?? trimmed
                let marker = capture(heading, group: 1, in: line) ?? "##"
                let level = Int(marker) ?? marker.count
                if index > 1 || title(label, fallback: "") != articleTitle {
                    appendHeading(label, level: level, to: contents, sections: &sections)
                }
            } else if setext || plainHeading {
                flushProse()
                if index > 1 || title(trimmed, fallback: "") != articleTitle {
                    appendHeading(trimmed, level: 2, to: contents, sections: &sections)
                }
                if setext { index += 1 }
            } else if index > 1 || title(trimmed, fallback: "") != articleTitle {
                let text = cleanProse(isGoDoc ? trimmed : line)
                if !text.trimmingCharacters(in: .whitespaces).isEmpty {
                    if isGoDoc {
                        if listItem.firstMatch(in: text, range: NSRange(text.startIndex..., in: text)) != nil {
                            flushProse()
                        }
                        wrappedProse.append(text)
                    } else {
                        appendInline(text, to: contents)
                    }
                }
            }
            previousBlank = false
        }
        flushProse()

        return ReaderArticle(contents: contents, sections: sections)
    }

    private static func capture(_ match: NSTextCheckingResult, group: Int, in text: String) -> String? {
        guard let range = Range(match.range(at: group), in: text) else { return nil }
        return String(text[range])
    }

    private static func cleanProse(_ text: String) -> String {
        let range = NSRange(text.startIndex..., in: text)
        let noImages = imageTag.stringByReplacingMatches(in: text, range: range, withTemplate: "")
        return imageMarkup.stringByReplacingMatches(in: noImages,
            range: NSRange(noImages.startIndex..., in: noImages), withTemplate: "$1")
    }

    private static func decodeEntities(_ text: String) -> String {
        [
            ("&lt;", "<"), ("&gt;", ">"), ("&quot;", "\""), ("&#39;", "'"),
            ("&apos;", "'"), ("&nbsp;", " "), ("&amp;", "&")
        ].reduce(text) { result, pair in result.replacingOccurrences(of: pair.0, with: pair.1) }
    }

    private static func cleanedInline(_ text: String) -> String {
        let replaced = inlineTags.stringByReplacingMatches(in: text,
            range: NSRange(text.startIndex..., in: text), withTemplate: "")
        return decodeEntities(replaced)
    }

    private static func paragraph(spacing: CGFloat = 0, before: CGFloat = 0, indent: CGFloat = 0) -> NSParagraphStyle {
        let style = NSMutableParagraphStyle()
        style.lineSpacing = 3
        style.paragraphSpacing = spacing
        style.paragraphSpacingBefore = before
        style.firstLineHeadIndent = indent
        style.headIndent = indent
        return style
    }

    private static func appendHeading(_ text: String, level: Int, to contents: NSMutableAttributedString, sections: inout [ReaderSection]) {
        let label = cleanedInline(cleanProse(text)).trimmingCharacters(in: .whitespaces)
        guard !label.isEmpty else { return }
        let range = NSRange(location: contents.length, length: label.utf16.count)
        let size: CGFloat = level <= 2 ? 20 : 16
        contents.append(NSAttributedString(string: label + "\n", attributes: [
            .font: NSFont.systemFont(ofSize: size, weight: .semibold),
            .foregroundColor: NSColor.labelColor,
            .paragraphStyle: paragraph(spacing: 4, before: 12)
        ]))
        if sections.count < 100 { sections.append(ReaderSection(title: label, target: .text(range))) }
    }

    private static func appendCode(_ text: String, to contents: NSMutableAttributedString) {
        contents.append(NSAttributedString(string: text + "\n", attributes: [
            .font: NSFont.monospacedSystemFont(ofSize: 12.5, weight: .regular),
            .foregroundColor: NSColor.labelColor,
            .paragraphStyle: paragraph(indent: 12)
        ]))
    }

    private static func appendInline(_ text: String, to contents: NSMutableAttributedString) {
        let source = text as NSString
        let style = paragraph()
        let regular: [NSAttributedString.Key: Any] = [
            .font: NSFont.systemFont(ofSize: 14),
            .foregroundColor: NSColor.labelColor,
            .paragraphStyle: style
        ]
        var position = 0
        for match in proseMarkup.matches(in: text, range: NSRange(location: 0, length: source.length)) {
            if match.range.location > position {
                let prefix = source.substring(with: NSRange(location: position, length: match.range.location - position))
                contents.append(NSAttributedString(string: cleanedInline(prefix), attributes: regular))
            }
            var attributes = regular
            let token: String
            if let code = capture(match, group: 1, in: text) {
                token = code
                attributes[.font] = NSFont.monospacedSystemFont(ofSize: 12.5, weight: .regular)
                attributes[.backgroundColor] = NSColor.quaternaryLabelColor.withAlphaComponent(0.09)
            } else if let bold = capture(match, group: 2, in: text) {
                token = cleanedInline(bold)
                attributes[.font] = NSFont.systemFont(ofSize: 14, weight: .semibold)
            } else if let label = capture(match, group: 3, in: text) ?? capture(match, group: 5, in: text) {
                token = cleanedInline(label)
                if let link = capture(match, group: 4, in: text) ?? capture(match, group: 6, in: text),
                   let url = safeExternalURL(link) {
                    attributes[.link] = url
                }
            } else {
                token = capture(match, group: 8, in: text) ?? ""
                switch capture(match, group: 7, in: text)?.lowercased() {
                case "code", "tt": attributes[.font] = NSFont.monospacedSystemFont(ofSize: 12.5, weight: .regular)
                case "b", "strong": attributes[.font] = NSFont.systemFont(ofSize: 14, weight: .semibold)
                case "i", "em": attributes[.font] = NSFontManager.shared.convert(NSFont.systemFont(ofSize: 14), toHaveTrait: .italicFontMask)
                default: break
                }
            }
            contents.append(NSAttributedString(string: token, attributes: attributes))
            position = NSMaxRange(match.range)
        }
        if position < source.length {
            contents.append(NSAttributedString(string: cleanedInline(source.substring(from: position)), attributes: regular))
        }
        contents.append(NSAttributedString(string: "\n", attributes: regular))
    }
}
