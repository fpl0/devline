import AppKit
import SwiftUI
import WebKit

struct DocReader: View {
    let document: Document
    let packName: String
    var anchor: String?
    var focus: String?
    var restoredPosition: Double?
    var restoredOffset: Double?
    var isSaved = false
    var save: (String?, String?) -> Void = { _, _ in }
    var navigate: (URL) -> Void = { NSWorkspace.shared.open($0) }
    var openPart: (String) -> Void = { _ in }
    var reportPosition: (Double, String?, Double?) -> Void = { _, _, _ in }
    @AppStorage("devline.readerScale") private var scale = 1.0
    @AppStorage("devline.contentsPinned") private var contentsPinned = false
    @AppStorage("devline.focusReading") private var focusReading = false
    @State private var article: ReaderArticle?
    @State private var currentAnchor: String?
    @State private var currentFocus: String?
    @State private var htmlSections: [ReaderSection] = []
    @State private var findVisible = false
    @State private var outlineVisible = false
    @State private var outlineFilter = ""
    @State private var outlineSelection: String?
    @State private var findText = ""
    @State private var request: ReaderRequest?
    @State private var matchFound: Bool?
    @FocusState private var findFocused: Bool
    @FocusState private var outlineFocused: Bool

    private var html: String? { document.html.flatMap { $0.isEmpty ? nil : $0 } }
    private var sections: [ReaderSection] { html == nil ? (article?.sections ?? []) : htmlSections }
    private var filteredSections: [ReaderSection] {
        sections.filter { outlineFilter.isEmpty || $0.title.localizedCaseInsensitiveContains(outlineFilter) }
    }
    private var initialTarget: ReaderTarget {
        ReaderTarget(anchor: anchor, focus: focus ?? document.focusText, position: restoredPosition, offset: restoredOffset)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            readerBar
            Divider()
            if findVisible { findBar; Divider() }
            HSplitView {
                articleBody.frame(minWidth: 300, maxWidth: .infinity, maxHeight: .infinity)
                if contentsPinned {
                    outlineContents(pinned: true)
                        .frame(minWidth: 180, idealWidth: 215, maxWidth: 260, maxHeight: .infinity)
                        .background(.quaternary.opacity(0.15))
                }
            }
            if document.previousId != nil || document.nextId != nil { continuationBar }
        }
        .task(id: initialTarget) {
            currentAnchor = initialTarget.anchor
            currentFocus = initialTarget.focus
            if html == nil, article == nil { article = ReaderMarkup.article(document, fallback: "\(packName) Overview") }
            if let position = initialTarget.position {
                request = ReaderRequest(action: .restore(position, anchor: initialTarget.anchor, offset: initialTarget.offset))
            } else if let anchor = initialTarget.anchor, !anchor.isEmpty, html != nil {
                request = ReaderRequest(action: .jump(ReaderSection(title: "", target: .anchor(anchor), fallbackText: initialTarget.focus)))
            } else if let focus = initialTarget.focus, !focus.isEmpty {
                let target = html != nil || article?.contents.string.localizedCaseInsensitiveContains(focus) == true
                    ? focus : ReaderMarkup.summary(focus)
                if !target.isEmpty { request = ReaderRequest(action: .focus(target)) }
            }
        }
    }

    @ViewBuilder private var articleBody: some View {
            if let html {
                OfflineArticle(html: html, title: ReaderMarkup.title(document.title, fallback: "\(packName) Overview"),
                               sourceURL: document.sourceUrl, scale: scale, request: request,
                               report: reportFind, reportSections: { htmlSections = $0 },
                               navigate: navigate, reportPosition: { fraction, anchor, offset, userInitiated in
                                   if userInitiated { currentAnchor = anchor; currentFocus = nil }
                                   reportPosition(fraction, anchor, offset)
                               })
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if let article {
                GeometryReader { geometry in
                    PlainArticle(article: article, horizontalInset: max(25, (geometry.size.width - 710 * scale) / 2),
                                 scale: scale, request: request, report: reportFind,
                                 reportPosition: { reportPosition($0, currentAnchor, nil) })
                }
            } else {
                ProgressView().frame(maxWidth: .infinity, maxHeight: .infinity)
            }
    }

    private var readerBar: some View {
        HStack(spacing: 12) {
            Label("\(packName) · \(displayDocVersion(document.version))", systemImage: "book")
                .font(.callout.weight(.medium)).foregroundStyle(.secondary).lineLimit(1)
                .help("\(packName) \(document.version) — installed offline")
            Spacer(minLength: 5)
            Button { focusReading.toggle() } label: { Image(systemName: focusReading ? "arrow.down.right.and.arrow.up.left" : "arrow.up.left.and.arrow.down.right") }
                .buttonStyle(.borderless).keyboardShortcut("r", modifiers: [.command, .shift])
                .help(focusReading ? "Exit Focus Reading (⇧⌘R)" : "Focus Reading (⇧⌘R)")
                .accessibilityLabel(focusReading ? "Exit Focus Reading" : "Focus Reading")
            Button { save(currentAnchor, currentFocus) } label: { Image(systemName: isSaved ? "bookmark.fill" : "bookmark") }
                .buttonStyle(.borderless).keyboardShortcut("d")
                .help(isSaved ? "Remove Saved Reference (⌘D)" : "Save Reference (⌘D)")
                .accessibilityLabel(isSaved ? "Remove Saved Reference" : "Save Reference")
            if !sections.isEmpty {
                Button {
                    outlineFilter = ""; outlineSelection = nil
                    if contentsPinned { outlineFocused = true } else { outlineVisible.toggle() }
                } label: { Label("Contents", systemImage: "list.bullet") }
                    .buttonStyle(.borderless).keyboardShortcut("o", modifiers: [.command, .shift])
                    .help("Search Page Contents (⇧⌘O)").accessibilityLabel("Search Page Contents")
                    .popover(isPresented: $outlineVisible, arrowEdge: .bottom) {
                        outlineContents(pinned: false).frame(width: 380, height: 430).task { outlineFocused = true }
                    }
            }
            Button { findVisible = true; findFocused = true } label: { Image(systemName: "magnifyingglass") }
                .buttonStyle(.borderless).keyboardShortcut("f", modifiers: [.command, .option])
                .help("Find in Page (⌥⌘F)").accessibilityLabel("Find in Page")
            Menu {
                Button("Larger Text") { scale = min(1.65, scale + 0.1) }.keyboardShortcut("+")
                Button("Smaller Text") { scale = max(0.8, scale - 0.1) }.keyboardShortcut("-")
                Button("Actual Size") { scale = 1 }.keyboardShortcut("0")
                Toggle("Pin Page Contents", isOn: $contentsPinned)
                Divider()
                Button("Copy Reference Link") { copyReferenceLink() }
                Button("Copy Official URL") { copy(document.sourceUrl) }
            } label: { Image(systemName: "textformat.size") }
                .menuStyle(.borderlessButton).fixedSize().accessibilityLabel("Reading Options")
            if let source = safeExternalURL(document.sourceUrl) {
                Link(destination: source) { Image(systemName: "arrow.up.right.square") }
                    .help("Open Official Source in Browser").accessibilityLabel("Open Official Source")
            }
        }.padding(.horizontal, 20).padding(.vertical, 10)
    }

    private func outlineContents(pinned: Bool) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack {
                Text("On This Page").font(.headline)
                Spacer()
                Button {
                    outlineVisible = false
                    contentsPinned.toggle()
                } label: { Image(systemName: pinned ? "pin.slash" : "pin") }
                    .buttonStyle(.borderless)
                    .help(pinned ? "Unpin Contents" : "Pin Contents Beside Article")
                    .accessibilityLabel(pinned ? "Unpin Contents" : "Pin Contents Beside Article")
            }
            TextField("Find a section or symbol", text: $outlineFilter)
                .textFieldStyle(.roundedBorder).focused($outlineFocused)
                .onSubmit { if let item = filteredSections.first(where: { $0.id == outlineSelection }) ?? filteredSections.first { jump(item) } }
                .onKeyPress(.downArrow) { moveOutline(1); return .handled }
                .onKeyPress(.upArrow) { moveOutline(-1); return .handled }
                .onKeyPress(.escape) { outlineVisible = false; return .handled }
            if filteredSections.isEmpty {
                Text("No matching sections").foregroundStyle(.secondary).padding(.vertical, 18)
            } else {
                ScrollViewReader { proxy in
                    ScrollView {
                        LazyVStack(alignment: .leading, spacing: 2) {
                            ForEach(filteredSections) { section in
                                Button { jump(section) } label: {
                                    Text(section.title).lineLimit(2).font(.callout)
                                        .padding(.leading, CGFloat(max(0, section.depth - 2)) * 10)
                                        .padding(.horizontal, 8).padding(.vertical, 7)
                                        .frame(maxWidth: .infinity, alignment: .leading)
                                        .background(outlineSelection == section.id ? Color.accentColor.opacity(0.15) : .clear,
                                                    in: RoundedRectangle(cornerRadius: 5))
                                        .contentShape(Rectangle())
                                }.buttonStyle(.plain).id(section.id)
                            }
                        }
                    }
                    .onChange(of: outlineSelection) { _, selected in if let selected { proxy.scrollTo(selected) } }
                }
            }
            Text("\(sections.count) sections · ↑ ↓ to select · Return to jump")
                .font(.caption).foregroundStyle(.secondary)
        }.padding(14)
    }

    private func moveOutline(_ offset: Int) {
        guard !filteredSections.isEmpty else { return }
        let index = filteredSections.firstIndex { $0.id == outlineSelection } ?? (offset > 0 ? -1 : filteredSections.count)
        outlineSelection = filteredSections[min(max(0, index + offset), filteredSections.count - 1)].id
    }

    private func jump(_ section: ReaderSection) {
        switch section.target {
        case .anchor(let anchor): currentAnchor = anchor; currentFocus = nil
        case .text: currentAnchor = nil; currentFocus = section.title
        }
        request = ReaderRequest(action: .jump(section))
        outlineVisible = false
    }

    private var continuationBar: some View {
        HStack {
            Button("Previous Part") { if let id = document.previousId { openPart(id) } }
                .disabled(document.previousId == nil)
            Spacer()
            if let part = document.part, let parts = document.parts {
                Text("Part \(part) of \(parts)").font(.caption).foregroundStyle(.secondary)
            }
            Spacer()
            Button("Continue Reading") { if let id = document.nextId { openPart(id) } }
                .disabled(document.nextId == nil)
        }.padding(.horizontal, 20).padding(.vertical, 10).background(.bar)
    }

    private var findBar: some View {
        HStack(spacing: 8) {
            TextField("Find in this page", text: $findText)
                .textFieldStyle(.roundedBorder).focused($findFocused)
                .onSubmit { searchPage() }
                .onChange(of: findText) { _, _ in searchPage(fromStart: true) }
                .onKeyPress(.escape) { closeFind(); return .handled }
            if matchFound == false { Text("No matches").font(.caption).foregroundStyle(.secondary) }
            Button { searchPage(backwards: true) } label: { Image(systemName: "chevron.up") }
                .keyboardShortcut("g", modifiers: [.command, .shift]).help("Previous Match (⇧⌘G)")
                .accessibilityLabel("Previous Match").disabled(findText.isEmpty)
            Button { searchPage() } label: { Image(systemName: "chevron.down") }
                .keyboardShortcut("g").help("Next Match (⌘G)")
                .accessibilityLabel("Next Match").disabled(findText.isEmpty)
            Button(action: closeFind) { Image(systemName: "xmark") }
                .buttonStyle(.borderless).help("Close Find").accessibilityLabel("Close Find")
        }.padding(.horizontal, 20).padding(.vertical, 8).background(.bar)
            .task { findFocused = true }
    }

    private func searchPage(backwards: Bool = false, fromStart: Bool = false) {
        matchFound = nil
        request = findText.isEmpty ? nil : ReaderRequest(action: .find(text: findText, backwards: backwards, fromStart: fromStart))
    }
    private func reportFind(_ current: ReaderRequest, found: Bool) {
        if request == current && findVisible { matchFound = found }
    }
    private func closeFind() { findVisible = false; findText = ""; request = nil; matchFound = nil }
    private func copy(_ text: String) { NSPasteboard.general.clearContents(); NSPasteboard.general.setString(text, forType: .string) }
    private func copyReferenceLink() {
        var components = URLComponents()
        components.scheme = "devline"
        components.host = "docs"
        components.queryItems = [URLQueryItem(name: "id", value: document.id)]
        if let anchor = currentAnchor { components.queryItems?.append(URLQueryItem(name: "anchor", value: anchor)) }
        if let focus = currentFocus { components.queryItems?.append(URLQueryItem(name: "focus", value: focus)) }
        if let address = components.string { copy(address) }
    }
    private struct ReaderTarget: Equatable { let anchor, focus: String?; let position, offset: Double? }
}

private struct ReaderArticle {
    let contents: NSAttributedString
    let sections: [ReaderSection]
}

private struct ReaderSection: Equatable, Identifiable {
    enum Target: Equatable {
        case text(NSRange)
        case anchor(String)
    }

    let title: String
    let target: Target
    var depth = 2
    var fallbackText: String?
    var id: String {
        switch target {
        case .text(let range): "text-\(range.location)"
        case .anchor(let id): id
        }
    }
}

private struct ReaderRequest: Equatable {
    enum Action: Equatable {
        case find(text: String, backwards: Bool, fromStart: Bool)
        case focus(String)
        case jump(ReaderSection)
        case restore(Double, anchor: String?, offset: Double?)
    }

    let id = UUID()
    let action: Action
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

    fileprivate static func article(_ document: Document, fallback: String) -> ReaderArticle {
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

private struct PlainArticle: NSViewRepresentable {
    let article: ReaderArticle
    let horizontalInset: CGFloat
    let scale: Double
    let request: ReaderRequest?
    let report: (ReaderRequest, Bool) -> Void
    let reportPosition: (Double) -> Void

    func makeCoordinator() -> Coordinator { Coordinator() }

    func makeNSView(context: Context) -> NSScrollView {
        let scrollView = NSTextView.scrollableTextView()
        scrollView.drawsBackground = false
        guard let textView = scrollView.documentView as? NSTextView else { return scrollView }
        textView.drawsBackground = false
        textView.isEditable = false
        textView.isSelectable = true
        textView.textContainerInset = NSSize(width: horizontalInset, height: 20)
        textView.setAccessibilityLabel("Document contents")
        textView.textStorage?.setAttributedString(scaledContents())
        context.coordinator.attach(scrollView, report: reportPosition)
        return scrollView
    }

    func updateNSView(_ scrollView: NSScrollView, context: Context) {
        guard let textView = scrollView.documentView as? NSTextView else { return }
        context.coordinator.reportPosition = reportPosition
        if context.coordinator.scale != scale {
            context.coordinator.scale = scale
            textView.textStorage?.setAttributedString(scaledContents())
        }
        if textView.textContainerInset.width != horizontalInset {
            textView.textContainerInset = NSSize(width: horizontalInset, height: 20)
        }
        guard context.coordinator.last != request else { return }
        let last = context.coordinator.last
        context.coordinator.last = request
        guard let request else {
            textView.setSelectedRange(NSRange(location: textView.selectedRange().location, length: 0))
            return
        }

        let coordinator = context.coordinator
        DispatchQueue.main.async {
            guard coordinator.last == request else { return }
            scrollView.layoutSubtreeIfNeeded()
            apply(request, last: last, to: textView, in: scrollView)
        }
    }

    private func apply(_ request: ReaderRequest, last: ReaderRequest?, to textView: NSTextView,
                       in scrollView: NSScrollView) {
        switch request.action {
        case .focus(let text):
            apply(ReaderRequest(action: .find(text: text, backwards: false, fromStart: true)), last: last, to: textView, in: scrollView)
        case .restore(let fraction, _, _):
            let height = max(0, textView.frame.height - scrollView.contentView.bounds.height)
            scrollView.contentView.scroll(to: NSPoint(x: 0, y: height * min(1, max(0, fraction))))
            scrollView.reflectScrolledClipView(scrollView.contentView)
        case .jump(let section):
            guard case .text(let range) = section.target else { return }
            textView.setSelectedRange(NSRange(location: range.location, length: 0))
            scrollTo(range, textView: textView, scrollView: scrollView)
        case .find(let text, let backwards, let fromStart):
            let contents = textView.string as NSString
            let whole = NSRange(location: 0, length: contents.length)
            let selection = textView.selectedRange()
            let next = min(NSMaxRange(selection), contents.length)
            var options: NSString.CompareOptions = [.caseInsensitive, .diacriticInsensitive]
            if backwards { options.insert(.backwards) }
            let range: NSRange
            if fromStart || last == nil { range = whole }
            else if backwards { range = NSRange(location: 0, length: selection.location) }
            else { range = NSRange(location: next, length: contents.length - next) }
            var found = contents.range(of: text, options: options, range: range)
            if found.location == NSNotFound && range != whole {
                found = contents.range(of: text, options: options, range: whole)
            }
            if found.location != NSNotFound {
                textView.setSelectedRange(found)
                if fromStart { scrollTo(found, textView: textView, scrollView: scrollView) }
                else { textView.scrollRangeToVisible(found) }
                textView.showFindIndicator(for: found)
            }
            report(request, found.location != NSNotFound)
        }
    }

    private func scrollTo(_ range: NSRange, textView: NSTextView, scrollView: NSScrollView) {
        textView.scrollRangeToVisible(range)
        guard let layout = textView.layoutManager, let container = textView.textContainer else { return }
        layout.ensureLayout(forCharacterRange: range)
        let glyphs = layout.glyphRange(forCharacterRange: range, actualCharacterRange: nil)
        let rect = layout.boundingRect(forGlyphRange: glyphs, in: container)
        let maxY = max(0, textView.frame.height - scrollView.contentView.bounds.height)
        let y = min(maxY, max(0, rect.minY + textView.textContainerOrigin.y - 25))
        scrollView.contentView.scroll(to: NSPoint(x: 0, y: y))
        scrollView.reflectScrolledClipView(scrollView.contentView)
    }

    private func scaledContents() -> NSAttributedString {
        let contents = NSMutableAttributedString(attributedString: article.contents)
        contents.enumerateAttribute(.font, in: NSRange(location: 0, length: contents.length)) { value, range, _ in
            if let font = value as? NSFont {
                contents.addAttribute(.font, value: NSFontManager.shared.convert(font, toSize: font.pointSize * scale), range: range)
            }
        }
        return contents
    }

    final class Coordinator {
        var last: ReaderRequest?
        var scale = 1.0
        var reportPosition: ((Double) -> Void)?
        private var observer: NSObjectProtocol?
        func attach(_ view: NSScrollView, report: @escaping (Double) -> Void) {
            reportPosition = report
            view.contentView.postsBoundsChangedNotifications = true
            observer = NotificationCenter.default.addObserver(forName: NSView.boundsDidChangeNotification,
                                                               object: view.contentView, queue: .main) { [weak self, weak view] _ in
                guard let view, let content = view.documentView else { return }
                let maximum = max(0, content.frame.height - view.contentView.bounds.height)
                self?.reportPosition?(maximum > 0 ? view.contentView.bounds.minY / maximum : 0)
            }
        }
        deinit { if let observer { NotificationCenter.default.removeObserver(observer) } }
    }
}

func safeExternalURL(_ value: String) -> URL? {
    guard let url = URL(string: value), let scheme = url.scheme?.lowercased(),
          scheme == "https" || scheme == "http", url.host != nil,
          url.user == nil, url.password == nil else { return nil }
    return url
}

private struct OfflineArticle: NSViewRepresentable {
    let html: String
    let title: String
    let sourceURL: String
    let scale: Double
    let request: ReaderRequest?
    let report: (ReaderRequest, Bool) -> Void
    let reportSections: ([ReaderSection]) -> Void
    let navigate: (URL) -> Void
    let reportPosition: (Double, String?, Double?, Bool) -> Void

    func makeCoordinator() -> Coordinator { Coordinator() }

    func makeNSView(context: Context) -> WKWebView {
        let configuration = WKWebViewConfiguration()
        configuration.defaultWebpagePreferences.allowsContentJavaScript = false
        configuration.websiteDataStore = .nonPersistent()
        configuration.userContentController.add(context.coordinator, contentWorld: Coordinator.readerWorld, name: "readerAction")
        let view = WKWebView(frame: .zero, configuration: configuration)
        view.navigationDelegate = context.coordinator
        view.setAccessibilityLabel("Document contents")
        return view
    }

    func updateNSView(_ view: WKWebView, context: Context) {
        let coordinator = context.coordinator
        coordinator.request = request
        coordinator.report = report
        coordinator.reportSections = reportSections
        coordinator.navigate = navigate
        coordinator.reportPosition = reportPosition
        coordinator.sourceURL = sourceURL
        if view.pageZoom != scale { view.pageZoom = scale }
        if coordinator.lastHTML != html || coordinator.lastTitle != title {
            coordinator.load(page, html: html, title: title, in: view)
        } else { coordinator.applyRequest(in: view) }
    }

    static func dismantleNSView(_ view: WKWebView, coordinator: Coordinator) {
        view.configuration.userContentController.removeScriptMessageHandler(forName: "readerAction", contentWorld: Coordinator.readerWorld)
        view.navigationDelegate = nil
    }

    private var page: String {
        let hasTitle = html.range(of: #"(?i)<h1\b"#, options: .regularExpression) != nil
        let escapedTitle = title.replacingOccurrences(of: "&", with: "&amp;")
            .replacingOccurrences(of: "<", with: "&lt;").replacingOccurrences(of: ">", with: "&gt;")
        let heading = hasTitle ? "" : "<h1 class=\"reader-title\">\(escapedTitle)</h1>"
        return """
        <!doctype html><html lang="en"><meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1">
        <meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src data:; style-src 'unsafe-inline'; base-uri 'none'; form-action 'none'">
        <style>
        :root { color-scheme: light dark; --line: color-mix(in srgb, CanvasText 18%, Canvas); --soft: color-mix(in srgb, CanvasText 6%, Canvas); }
        * { box-sizing: border-box; }
        body { margin: 0; background: Canvas; color: CanvasText; font: 14px/1.65 -apple-system, system-ui, sans-serif; }
        main { max-inline-size: 800px; margin-inline: auto; padding: 24px 28px 48px; overflow-wrap: anywhere; }
        h1, h2, h3, h4, h5, h6, dt, section[id] { scroll-margin-block-start: 24px; }
        h1,h2,h3,h4,h5,h6 { margin: 28px 0 11px; line-height: 1.3; }
        h1 { font-size: 28px; font-weight: 650; } h2 { font-size: 21px; font-weight: 650; border-block-start: 1px solid var(--line); padding-block-start: 20px; }
        h3 { font-size: 17px; font-weight: 650; } h4,h5,h6 { font-size: 15px; font-weight: 650; }
        main > :first-child { margin-block-start: 0; }
        p,ul,ol,dl,blockquote,pre,.table-scroll,.code-region { margin: 0 0 15px; }
        ul,ol { padding-inline-start: 1.6em; } li + li { margin-block-start: 5px; } li > p { margin-block-end: 5px; }
        li > ul,li > ol { margin-block: 5px 0; }
        dt { font-weight: 600; margin-block: 18px 8px; } dt[id] { font: 600 13px/1.6 ui-monospace, Menlo, monospace; background: var(--soft); padding: 10px 12px; border-radius: 5px; }
        dd { margin-inline: 0; padding-inline-start: 14px; } dl.field-list { display: grid; grid-template-columns: auto 1fr; gap: 8px 14px; }
        blockquote,.admonition,.note,.warning,.tip,.important,.caution { padding: 10px 14px; border-inline-start: 3px solid var(--line); background: var(--soft); border-radius: 3px; margin-block: 16px; }
        .admonition-title { font-weight: 650; } .warning,.caution { border-color: #b57913; }
        code,kbd,samp { font: 0.91em ui-monospace, Menlo, monospace; background: var(--soft); padding: 1px 4px; border-radius: 4px; white-space: break-spaces; }
        pre,.table-scroll { max-inline-size: 100%; overflow-x: auto; overscroll-behavior-inline: contain; }
        pre { background: var(--soft); border: 1px solid var(--line); border-radius: 7px; padding: 35px 14px 14px; font: 12.5px/1.58 ui-monospace, Menlo, monospace; tab-size: 4; white-space: pre; overflow-wrap: normal; }
        pre code { font: inherit; background: none; padding: 0; white-space: inherit; }
        .code-region { position: relative; } .code-region pre { margin: 0; }
        .code-copy { position: absolute; inset-block-start: 7px; inset-inline-end: 9px; border: 0; border-radius: 4px; padding: 3px 7px; font: 11px -apple-system, system-ui; background: Canvas; color: CanvasText; cursor: pointer; }
        .code-copy:hover { background: var(--line); }
        table { min-inline-size: 100%; border-collapse: collapse; font-size: 13px; }
        th,td { padding: 8px 11px; text-align: start; vertical-align: top; border-block-end: 1px solid var(--line); }
        th { background: var(--soft); font-weight: 650; } td code,th code { white-space: nowrap; }
        hr { border: 0; border-block-start: 1px solid var(--line); margin-block: 24px; }
        img { max-inline-size: 100%; height: auto; } figure { margin-inline: 0; } figcaption { font-size: 12px; opacity: .8; }
        a { color: LinkText; text-underline-offset: 2px; } a:hover { text-decoration-thickness: 2px; }
        summary { cursor: pointer; font-weight: 600; padding-block: 7px; } details > :not(summary) { margin-inline-start: 12px; }
        .reference-target { box-shadow: inset 3px 0 0 AccentColor; padding-inline-start: 12px; border-radius: 3px; }
        :focus-visible { outline: 2px solid -webkit-focus-ring-color; outline-offset: 2px; }
        .kw,.keyword,.k { color: light-dark(#7b2b91, #d69de5); } .s,.string,.str { color: light-dark(#976019, #e6bd85); }
        </style><body><main id="devline-reader">\(heading)\(html)</main></body></html>
        """
    }

    final class Coordinator: NSObject, WKNavigationDelegate, WKScriptMessageHandler {
        static let readerWorld = WKContentWorld.world(name: "DevlineArticle")
        private(set) var lastHTML: String?
        private(set) var lastTitle: String?
        var sourceURL = ""
        var request: ReaderRequest?
        var report: ((ReaderRequest, Bool) -> Void)?
        var reportSections: (([ReaderSection]) -> Void)?
        var navigate: ((URL) -> Void)?
        var reportPosition: ((Double, String?, Double?, Bool) -> Void)?
        private var navigation: WKNavigation?
        private var ready = false
        private var lastRequest: ReaderRequest?

        func load(_ page: String, html: String, title: String, in webView: WKWebView) {
            lastHTML = html; lastTitle = title; ready = false; lastRequest = nil
            navigation = webView.loadHTMLString(page, baseURL: nil)
        }

        func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
            guard navigation === self.navigation else { return }
            webView.callAsyncJavaScript("""
                const article = document.getElementById('devline-reader');
                if (!article) return [];
                const send = value => window.webkit.messageHandlers.readerAction.postMessage(value);
                for (const table of article.querySelectorAll('table')) {
                    const region = document.createElement('div');
                    region.className = 'table-scroll'; region.tabIndex = 0;
                    region.setAttribute('role', 'region'); region.setAttribute('aria-label', 'Table');
                    table.before(region); region.append(table);
                }
                for (const block of article.querySelectorAll('pre')) {
                    block.tabIndex = 0; block.setAttribute('aria-label', 'Code block');
                    const region = document.createElement('div'); region.className = 'code-region';
                    block.before(region); region.append(block);
                    const button = document.createElement('button'); button.className = 'code-copy';
                    button.type = 'button'; button.textContent = 'Copy'; button.setAttribute('aria-label', 'Copy code');
                    button.addEventListener('click', () => {
                        send({action: 'copy', text: block.textContent});
                        button.textContent = 'Copied';
                        setTimeout(() => { button.textContent = 'Copy'; }, 1400);
                    });
                    region.append(button);
                }
                const sections = [];
                const seen = new Set();
                for (const heading of article.querySelectorAll('h1,h2,h3,h4,h5,h6,dt[id]')) {
                    const title = heading.textContent.replace(/[¶#]$/g, '').trim().replace(/\\s+/g, ' ');
                    if (!title || heading.classList.contains('reader-title')) continue;
                    let id = heading.id || heading.closest('section[id]')?.id;
                    if (!id) { id = 'devline-heading-' + sections.length; heading.id = id; }
                    if (seen.has(id)) { id = 'devline-heading-' + sections.length; heading.id = id; }
                    seen.add(id);
                    sections.push({id, title, depth: heading.tagName === 'DT' ? 4 : Number(heading.tagName.slice(1))});
                }
                const positions = sections.map(section => document.getElementById(section.id)).filter(Boolean);
                window.devlineUserScroll = false;
                for (const type of ['wheel', 'touchstart', 'pointerdown']) document.addEventListener(type, () => { window.devlineUserScroll = true; }, {passive: true});
                document.addEventListener('keydown', event => {
                    if (['ArrowUp','ArrowDown','PageUp','PageDown','Home','End',' '].includes(event.key)) window.devlineUserScroll = true;
                });
                let pending = false, semanticPosition = null, previousWidth = window.innerWidth;
                window.addEventListener('resize', () => {
                    if (window.innerWidth !== previousWidth && semanticPosition) {
                        const target = document.getElementById(semanticPosition.anchor);
                        if (target) window.scrollBy(0, target.getBoundingClientRect().top - semanticPosition.offset);
                    }
                    previousWidth = window.innerWidth;
                });
                document.addEventListener('scroll', () => {
                    if (pending) return;
                    pending = true;
                    setTimeout(() => {
                        pending = false;
                        const max = document.documentElement.scrollHeight - window.innerHeight;
                        const visible = positions.filter(node => node.getClientRects().length);
                        let low = 0, high = visible.length - 1, active = null;
                        while (low <= high) {
                            const middle = Math.floor((low + high) / 2);
                            if (visible[middle].getBoundingClientRect().top <= 70) { active = visible[middle].id; low = middle + 1; }
                            else high = middle - 1;
                        }
                        const offset = active ? document.getElementById(active).getBoundingClientRect().top : null;
                        semanticPosition = active ? {anchor: active, offset} : null;
                        send({action: 'position', fraction: max > 0 ? window.scrollY / max : 0, anchor: active, offset, userInitiated: window.devlineUserScroll});
                    }, 120);
                }, {passive: true});
                return sections;
                """, in: nil, in: Self.readerWorld) { [weak self, weak webView] result in
                    guard let self, let webView, navigation === self.navigation else { return }
                    let entries = (try? result.get()) as? [[String: Any]] ?? []
                    self.reportSections?(entries.compactMap { entry in
                        guard let id = entry["id"] as? String, let title = entry["title"] as? String else { return nil }
                        return ReaderSection(title: title, target: .anchor(id), depth: entry["depth"] as? Int ?? 2)
                    })
                    self.ready = true
                    self.applyRequest(in: webView)
                }
        }

        func userContentController(_ userContentController: WKUserContentController, didReceive message: WKScriptMessage) {
            guard message.name == "readerAction", message.frameInfo.isMainFrame,
                  let value = message.body as? [String: Any], let action = value["action"] as? String else { return }
            switch action {
            case "copy":
                guard let text = value["text"] as? String, text.utf8.count <= 2_000_000 else { return }
                NSPasteboard.general.clearContents(); NSPasteboard.general.setString(text, forType: .string)
            case "position":
                if let fraction = value["fraction"] as? Double, fraction.isFinite {
                    let offset = value["offset"] as? Double
                    reportPosition?(min(1, max(0, fraction)), value["anchor"] as? String, offset?.isFinite == true ? offset : nil, value["userInitiated"] as? Bool ?? false)
                }
            default: break
            }
        }

        func applyRequest(in webView: WKWebView) {
            guard ready, request != lastRequest else { return }
            lastRequest = request
            guard let request else {
                webView.evaluateJavaScript("window.getSelection()?.removeAllRanges()", in: nil, in: Self.readerWorld)
                return
            }
            webView.evaluateJavaScript("window.devlineUserScroll = false", in: nil, in: Self.readerWorld)
            switch request.action {
            case .focus(let text):
                webView.callAsyncJavaScript("""
                    const normalize = value => value.replace(/\\s+/g, '').toLowerCase();
                    const needle = normalize(text);
                    const target = [...document.querySelectorAll('#devline-reader h1,#devline-reader h2,#devline-reader h3,#devline-reader h4,#devline-reader dt[id],#devline-reader pre')]
                        .find(node => normalize(node.textContent).includes(needle));
                    if (!target || !needle) return false;
                    for (let node = target.parentElement; node; node = node.parentElement) if (node.tagName === 'DETAILS') node.open = true;
                    target.classList.add('reference-target');
                    target.scrollIntoView({block: 'start'});
                    return true;
                    """, arguments: ["text": text], in: nil, in: Self.readerWorld) { [weak self, weak webView] result in
                    guard let self, let webView, self.lastRequest == request else { return }
                    if (try? result.get()) as? Bool == true { self.report?(request, true) }
                    else { self.find(text, backwards: false, request: request, in: webView) }
                }
            case .restore(let position, let anchor, let offset):
                webView.callAsyncJavaScript("""
                    const target = anchor ? document.getElementById(anchor) : null;
                    if (target && offset !== null) window.scrollBy(0, target.getBoundingClientRect().top - offset);
                    else window.scrollTo(0, Math.max(0, document.documentElement.scrollHeight - window.innerHeight) * fraction);
                    """, arguments: ["fraction": min(1, max(0, position)), "anchor": anchor as Any? ?? NSNull(),
                                       "offset": offset as Any? ?? NSNull()], in: nil, in: Self.readerWorld)
            case .jump(let section):
                guard case .anchor(let id) = section.target else { return }
                webView.callAsyncJavaScript("""
                    const target = document.getElementById(anchor) || [...document.getElementsByName(anchor)][0];
                    if (target) {
                        for (let node = target.parentElement; node; node = node.parentElement) if (node.tagName === 'DETAILS') node.open = true;
                        target.scrollIntoView({block: 'start'});
                    }
                    return !!target;
                    """, arguments: ["anchor": id], in: nil, in: Self.readerWorld) { [weak self, weak webView] result in
                    guard let self, let webView, self.lastRequest == request,
                          (try? result.get()) as? Bool != true, let text = section.fallbackText, !text.isEmpty else { return }
                    self.find(text, backwards: false, request: request, in: webView)
                }
            case .find(let text, let backwards, let fromStart):
                if fromStart {
                    webView.callAsyncJavaScript("""
                        const article = document.getElementById('devline-reader');
                        const selection = window.getSelection();
                        if (!article || !selection) return;
                        const range = document.createRange(); range.selectNodeContents(article); range.collapse(true);
                        selection.removeAllRanges(); selection.addRange(range);
                        """, in: nil, in: Self.readerWorld) { [weak self, weak webView] _ in
                            guard let self, let webView, self.lastRequest == request else { return }
                            self.find(text, backwards: backwards, request: request, in: webView)
                        }
                } else { find(text, backwards: backwards, request: request, in: webView) }
            }
        }

        private func find(_ text: String, backwards: Bool, request: ReaderRequest, in webView: WKWebView) {
            let configuration = WKFindConfiguration(); configuration.wraps = true; configuration.backwards = backwards
            webView.find(text, configuration: configuration) { [weak self] result in
                guard self?.lastRequest == request else { return }
                self?.report?(request, result.matchFound)
            }
        }

        func webView(_ webView: WKWebView, decidePolicyFor action: WKNavigationAction,
                     decisionHandler: @escaping (WKNavigationActionPolicy) -> Void) {
            guard let url = action.request.url else { decisionHandler(.cancel); return }
            let address = url.absoluteString
            if address == "about:blank", action.navigationType != .linkActivated {
                decisionHandler(.allow); return
            }
            decisionHandler(.cancel)
            guard action.navigationType == .linkActivated else { return }
            if address.hasPrefix("about:blank#"), let fragment = url.fragment {
                var components = URLComponents(string: sourceURL)
                components?.fragment = fragment.removingPercentEncoding ?? fragment
                if let resolved = components?.url { navigate?(resolved) }
            } else if let safe = safeExternalURL(address) { navigate?(safe) }
        }
    }
}
