import AppKit
import SwiftUI
import WebKit

struct DocReader: View {
    let document: Document
    let packName: String
    @State private var article: ReaderArticle?
    @State private var htmlSections: [ReaderSection] = []
    @State private var findVisible = false
    @State private var findText = ""
    @State private var request: ReaderRequest?
    @State private var matchFound: Bool?
    @FocusState private var findFocused: Bool

    private var html: String? { document.html.flatMap { $0.isEmpty ? nil : $0 } }
    private var sections: [ReaderSection] { html == nil ? (article?.sections ?? []) : htmlSections }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            readerBar
            Divider()
            if findVisible { findBar; Divider() }
            if let html {
                OfflineArticle(
                    html: html,
                    title: ReaderMarkup.title(document.title, fallback: "\(packName) Overview"),
                    request: request,
                    report: reportFind,
                    reportSections: { htmlSections = $0 }
                )
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if let article {
                GeometryReader { geometry in
                    PlainArticle(
                        article: article,
                        horizontalInset: max(25, (geometry.size.width - 710) / 2),
                        request: request,
                        report: reportFind
                    )
                }
            } else {
                ProgressView().frame(maxWidth: .infinity, maxHeight: .infinity)
            }
        }
        .task(id: document.focusText) {
            if html == nil, article == nil {
                article = ReaderMarkup.article(document, fallback: "\(packName) Overview")
            }
            if let focus = document.focusText, !focus.isEmpty {
                let target = html != nil || article?.contents.string.localizedCaseInsensitiveContains(focus) == true
                    ? focus : ReaderMarkup.summary(focus)
                if !target.isEmpty {
                    request = ReaderRequest(action: .find(text: target, backwards: false, fromStart: true))
                }
            }
        }
    }

    private var readerBar: some View {
        HStack(spacing: 13) {
            Label("\(packName) · \(document.version)", systemImage: "book")
                .font(.callout.weight(.medium)).foregroundStyle(.secondary).lineLimit(1)
            Spacer(minLength: 5)
            if !sections.isEmpty {
                Menu {
                    ForEach(sections) { section in
                        Button(section.title) { request = ReaderRequest(action: .jump(section)) }
                    }
                } label: {
                    Label("On This Page", systemImage: "list.bullet")
                }
                .menuStyle(.borderlessButton)
                .fixedSize()
                .accessibilityLabel("On This Page")
            }
            Button {
                findVisible = true
                findFocused = true
            } label: {
                Image(systemName: "magnifyingglass")
            }
            .buttonStyle(.borderless)
            .keyboardShortcut("f", modifiers: [.command, .option])
            .help("Find in Page (⌥⌘F)")
            .accessibilityLabel("Find in Page")
            if let source = safeExternalURL(document.sourceUrl) {
                Link(destination: source) { Label("Source", systemImage: "arrow.up.right") }
                    .font(.callout)
                    .help("Open official source in your browser")
                    .accessibilityLabel("Open Official Source")
            }
        }
        .padding(.horizontal, 20)
        .padding(.vertical, 10)
    }

    private var findBar: some View {
        HStack(spacing: 8) {
            TextField("Find in this page", text: $findText)
                .textFieldStyle(.roundedBorder)
                .focused($findFocused)
                .onSubmit { searchPage() }
                .onChange(of: findText) { _, _ in searchPage(fromStart: true) }
                .onKeyPress(.escape) { closeFind(); return .handled }
            if matchFound == false {
                Text("No matches").font(.caption).foregroundStyle(.secondary)
            }
            Button { searchPage(backwards: true) } label: { Image(systemName: "chevron.up") }
                .keyboardShortcut("g", modifiers: [.command, .shift])
                .help("Previous Match (⇧⌘G)")
                .accessibilityLabel("Previous Match")
                .disabled(findText.isEmpty)
            Button { searchPage() } label: { Image(systemName: "chevron.down") }
                .keyboardShortcut("g")
                .help("Next Match (⌘G)")
                .accessibilityLabel("Next Match")
                .disabled(findText.isEmpty)
            Button(action: closeFind) { Image(systemName: "xmark") }
                .buttonStyle(.borderless)
                .help("Close Find")
                .accessibilityLabel("Close Find")
        }
        .padding(.horizontal, 20)
        .padding(.vertical, 8)
        .background(.bar)
        .task { findFocused = true }
    }

    private func searchPage(backwards: Bool = false, fromStart: Bool = false) {
        matchFound = nil
        request = findText.isEmpty ? nil : ReaderRequest(action: .find(text: findText, backwards: backwards, fromStart: fromStart))
    }

    private func reportFind(_ current: ReaderRequest, found: Bool) {
        if request == current && findVisible { matchFound = found }
    }

    private func closeFind() {
        findVisible = false
        findText = ""
        request = nil
        matchFound = nil
    }
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
        case jump(ReaderSection)
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
    let request: ReaderRequest?
    let report: (ReaderRequest, Bool) -> Void

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
        textView.textStorage?.setAttributedString(article.contents)
        return scrollView
    }

    func updateNSView(_ scrollView: NSScrollView, context: Context) {
        guard let textView = scrollView.documentView as? NSTextView else { return }
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

    final class Coordinator {
        var last: ReaderRequest?
    }
}

private func safeExternalURL(_ value: String) -> URL? {
    guard let url = URL(string: value), let scheme = url.scheme?.lowercased(),
          scheme == "https" || scheme == "http", url.host != nil,
          url.user == nil, url.password == nil else { return nil }
    return url
}

private struct OfflineArticle: NSViewRepresentable {
    let html: String
    let title: String
    let request: ReaderRequest?
    let report: (ReaderRequest, Bool) -> Void
    let reportSections: ([ReaderSection]) -> Void

    func makeCoordinator() -> Coordinator { Coordinator() }

    func makeNSView(context: Context) -> WKWebView {
        let config = WKWebViewConfiguration()
        config.defaultWebpagePreferences.allowsContentJavaScript = false
        config.websiteDataStore = .nonPersistent()
        let view = WKWebView(frame: .zero, configuration: config)
        view.navigationDelegate = context.coordinator
        view.setAccessibilityLabel("Document contents")
        return view
    }

    func updateNSView(_ view: WKWebView, context: Context) {
        let coordinator = context.coordinator
        coordinator.request = request
        coordinator.report = report
        coordinator.reportSections = reportSections
        if coordinator.lastHTML != html || coordinator.lastTitle != title {
            coordinator.load(page, html: html, title: title, in: view)
        } else {
            coordinator.applyRequest(in: view)
        }
    }

    private var page: String {
        let hasTitle = html.range(of: #"(?i)\A\s*<h1\b"#, options: .regularExpression) != nil
        let escapedTitle = title.replacingOccurrences(of: "&", with: "&amp;")
            .replacingOccurrences(of: "<", with: "&lt;")
            .replacingOccurrences(of: ">", with: "&gt;")
        let heading = hasTitle ? "" : "<h1 class=\"reader-title\">\(escapedTitle)</h1>"
        return """
            <!doctype html>
            <html lang="en">
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src data:; style-src 'unsafe-inline'; base-uri 'none'; form-action 'none'">
            <style>
            :root {
                color-scheme: light dark;
                --line: color-mix(in srgb, CanvasText 19%, Canvas);
                --soft: color-mix(in srgb, CanvasText 7%, Canvas);
            }
            * { box-sizing: border-box; }
            body { margin: 0; background: Canvas; color: CanvasText; font: 14px/1.62 -apple-system, system-ui, sans-serif; }
            main { max-inline-size: 760px; margin-inline: auto; padding: 24px 26px 38px; overflow-wrap: anywhere; }
            h1, h2, h3, h4, h5, h6 { margin: 24px 0 9px; line-height: 1.26; scroll-margin-block-start: 20px; }
            h1 { font-size: 29px; font-weight: 650; }
            h2 { font-size: 21px; font-weight: 650; }
            h3 { font-size: 17px; font-weight: 650; }
            h4, h5, h6 { font-size: 15px; font-weight: 650; }
            main > :first-child { margin-block-start: 0; }
            p, ul, ol, dl, blockquote, pre, .table-scroll { margin: 0 0 13px; }
            ul, ol { padding-inline-start: 1.6em; }
            li + li { margin-block-start: 4px; }
            li > p { margin-block-end: 5px; }
            li > ul, li > ol { margin-block: 4px 0; }
            dd { margin-inline-start: 1.4em; }
            blockquote { padding-inline-start: 13px; border-inline-start: 2px solid var(--line); }
            code, kbd { font: 0.91em ui-monospace, Menlo, monospace; background: var(--soft); padding: 1px 4px; border-radius: 4px; white-space: break-spaces; }
            pre, .table-scroll { max-inline-size: 100%; overflow-x: auto; overscroll-behavior-inline: contain; }
            pre { background: var(--soft); border-radius: 7px; padding: 12px 14px; font: 12.5px/1.54 ui-monospace, Menlo, monospace; tab-size: 4; white-space: pre; overflow-wrap: normal; }
            pre code { font: inherit; background: none; padding: 0; white-space: inherit; }
            table { min-inline-size: 100%; border-collapse: collapse; font-size: 13px; }
            th, td { padding: 7px 10px; text-align: start; vertical-align: top; border-block-end: 1px solid var(--line); }
            th { background: var(--soft); font-weight: 650; }
            td code, th code { white-space: nowrap; }
            hr { border: 0; border-block-start: 1px solid var(--line); margin-block: 20px; }
            img { max-inline-size: 100%; height: auto; }
            a { color: LinkText; text-underline-offset: 2px; }
            :focus-visible { outline: 2px solid -webkit-focus-ring-color; outline-offset: 2px; }
            </style>
            <body><main id="devline-reader">\(heading)\(html)</main></body>
            </html>
            """
    }

    final class Coordinator: NSObject, WKNavigationDelegate {
        private static let readerWorld = WKContentWorld.world(name: "DevlineArticle")
        private(set) var lastHTML: String?
        private(set) var lastTitle: String?
        var request: ReaderRequest?
        var report: ((ReaderRequest, Bool) -> Void)?
        var reportSections: (([ReaderSection]) -> Void)?
        private var navigation: WKNavigation?
        private var ready = false
        private var lastRequest: ReaderRequest?

        func load(_ page: String, html: String, title: String, in webView: WKWebView) {
            lastHTML = html
            lastTitle = title
            ready = false
            lastRequest = nil
            navigation = webView.loadHTMLString(page, baseURL: nil)
        }

        func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
            guard navigation === self.navigation else { return }
            webView.callAsyncJavaScript("""
                const article = document.getElementById('devline-reader');
                if (!article) return [];
                for (const table of article.querySelectorAll('table')) {
                    const region = document.createElement('div');
                    region.className = 'table-scroll';
                    region.tabIndex = 0;
                    region.setAttribute('role', 'region');
                    region.setAttribute('aria-label', 'Table');
                    table.before(region);
                    region.append(table);
                }
                for (const block of article.querySelectorAll('pre')) {
                    block.tabIndex = 0;
                    block.setAttribute('aria-label', 'Code block');
                }
                const sections = [];
                for (const heading of article.querySelectorAll('h1, h2, h3, h4, h5, h6')) {
                    const title = heading.textContent.trim();
                    if (!title || heading.classList.contains('reader-title')) continue;
                    if (!heading.id) {
                        let suffix = sections.length;
                        while (document.getElementById('devline-heading-' + suffix)) suffix++;
                        heading.id = 'devline-heading-' + suffix;
                    }
                    sections.push({ id: heading.id, title });
                    if (sections.length === 100) break;
                }
                return sections;
                """, in: nil, in: Self.readerWorld) { [weak self, weak webView] result in
                    guard let self, let webView, navigation === self.navigation else { return }
                    let entries = (try? result.get()) as? [[String: String]] ?? []
                    self.reportSections?(entries.compactMap { entry in
                        guard let id = entry["id"], let title = entry["title"] else { return nil }
                        return ReaderSection(title: title, target: .anchor(id))
                    })
                    self.ready = true
                    self.applyRequest(in: webView)
                }
        }

        func applyRequest(in webView: WKWebView) {
            guard ready, request != lastRequest else { return }
            lastRequest = request
            guard let request else {
                webView.evaluateJavaScript("window.getSelection()?.removeAllRanges()", in: nil, in: Self.readerWorld)
                return
            }
            switch request.action {
            case .jump(let section):
                guard case .anchor(let id) = section.target else { return }
                webView.callAsyncJavaScript(
                    "document.getElementById(anchor)?.scrollIntoView({block: 'start'});",
                    arguments: ["anchor": id],
                    in: nil, in: Self.readerWorld
                )
            case .find(let text, let backwards, let fromStart):
                if fromStart {
                    webView.callAsyncJavaScript("""
                        const article = document.getElementById('devline-reader');
                        const selection = window.getSelection();
                        if (!article || !selection) return;
                        const range = document.createRange();
                        range.selectNodeContents(article);
                        range.collapse(true);
                        selection.removeAllRanges();
                        selection.addRange(range);
                        """, in: nil, in: Self.readerWorld) { [weak self, weak webView] _ in
                            guard let self, let webView, self.lastRequest == request else { return }
                            self.find(text, backwards: backwards, request: request, in: webView)
                        }
                } else {
                    find(text, backwards: backwards, request: request, in: webView)
                }
            }
        }

        private func find(_ text: String, backwards: Bool, request: ReaderRequest, in webView: WKWebView) {
            let configuration = WKFindConfiguration()
            configuration.wraps = true
            configuration.backwards = backwards
            webView.find(text, configuration: configuration) { [weak self] result in
                guard self?.lastRequest == request else { return }
                self?.report?(request, result.matchFound)
            }
        }

        func webView(_ webView: WKWebView, decidePolicyFor action: WKNavigationAction,
                     decisionHandler: @escaping (WKNavigationActionPolicy) -> Void) {
            guard let url = action.request.url else { decisionHandler(.cancel); return }
            let address = url.absoluteString
            if address == "about:blank" || address.hasPrefix("about:blank#") {
                decisionHandler(.allow)
            } else {
                if action.navigationType == .linkActivated, let external = safeExternalURL(address) {
                    NSWorkspace.shared.open(external)
                }
                decisionHandler(.cancel)
            }
        }
    }
}
