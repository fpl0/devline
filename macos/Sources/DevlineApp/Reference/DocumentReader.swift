import AppKit
import SwiftUI

struct DocReader: View {
    let document: Document
    let pack: DocPack?
    let packName: String
    var anchor: String?
    var focus: String?
    var restoredPosition: Double?
    var restoredOffset: Double?
    var cueSearchMatch = false
    var isSaved = false
    var save: (String?, String?) -> Void = { _, _ in }
    let linkRevision: String
    let linkStatuses: ([String]) async throws -> [DocLinkStatus]
    let navigate: (String, DocLinkStatus?) -> Void
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
    @State private var sourceVisible = false
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
    private var imported: Bool { document.packId.hasPrefix("devdocs-") }
    private var source: URL? { safeExternalURL(document.sourceUrl) }
    private var sourceLabel: String {
        guard imported else { return "Open Publisher Site" }
        return source?.host == "devdocs.io" ? "Open DevDocs" : "Open Publisher Site"
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
                request = ReaderRequest(action: .jump(ReaderSection(title: "", target: .anchor(anchor), fallbackText: initialTarget.focus)), cue: cueSearchMatch)
            } else if let focus = initialTarget.focus, !focus.isEmpty {
                let target = html != nil || article?.contents.string.localizedCaseInsensitiveContains(focus) == true
                    ? focus : ReaderMarkup.summary(focus)
                if !target.isEmpty { request = ReaderRequest(action: .focus(target), cue: cueSearchMatch) }
            }
        }
    }

    @ViewBuilder private var articleBody: some View {
            if let html {
                OfflineArticle(html: html, title: ReaderMarkup.title(document.title, fallback: "\(packName) Overview"),
                               sourceURL: document.sourceUrl, packID: document.packId, scale: scale, request: request,
                               report: reportFind, reportSections: { htmlSections = $0 },
                               linkRevision: linkRevision, linkStatuses: linkStatuses, navigate: navigate,
                               reportPosition: { fraction, anchor, offset, userInitiated in
                                   if userInitiated { currentAnchor = anchor; currentFocus = nil }
                                   reportPosition(fraction, anchor, offset)
                               })
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if let article {
                GeometryReader { geometry in
                    PlainArticle(article: article, horizontalInset: max(25, (geometry.size.width - 710 * scale) / 2),
                                 scale: scale, request: request, report: reportFind, navigate: { navigate($0, nil) },
                                 reportPosition: { reportPosition($0, currentAnchor, nil) })
                }
            } else {
                ProgressView().frame(maxWidth: .infinity, maxHeight: .infinity)
            }
    }

    private var readerBar: some View {
        ViewThatFits(in: .horizontal) {
            readerBar(showActionLabels: true)
            readerBar(showActionLabels: false)
        }
    }

    private func readerBar(showActionLabels: Bool) -> some View {
        HStack(spacing: 12) {
            Label("\(packName) · \(displayDocVersion(document.version))", systemImage: "book")
                .font(.callout.weight(.medium)).foregroundStyle(.secondary).lineLimit(1)
                .help("\(packName) · Installed")
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
                } label: {
                    if showActionLabels { Label("Contents", systemImage: "list.bullet").fixedSize() }
                    else { Image(systemName: "list.bullet") }
                }
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
            } label: { Image(systemName: "textformat.size") }
                .menuStyle(.borderlessButton).fixedSize().accessibilityLabel("Reading Options")
            Button { sourceVisible.toggle() } label: {
                if showActionLabels { Label("Source", systemImage: "info.circle").fixedSize() }
                else { Image(systemName: "info.circle") }
            }
            .buttonStyle(.borderless).accessibilityLabel("Source Details").help("Source Details")
            .popover(isPresented: $sourceVisible) { sourceDetails }
        }.padding(.horizontal, 20).padding(.vertical, 10)
    }

    private var sourceDetails: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text(packName).font(.headline)
            Text(imported ? "Imported via DevDocs" : pack?.localOnly == true ? "Prepared from the local SDK" : "Publisher reference")
                .foregroundStyle(.secondary)
            Text(document.version).font(.caption.monospaced()).textSelection(.enabled)
            if let date = pack?.snapshotDate { Text("Snapshot \(date)").font(.caption).foregroundStyle(.secondary) }
            if let source {
                HStack {
                    Link(sourceLabel + " ↗", destination: source)
                    Button("Copy URL") { copyToPasteboard(source.absoluteString) }.buttonStyle(.borderless)
                }
            }
        }.padding(16).frame(width: 340, alignment: .leading)
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
    private func copyReferenceLink() {
        var components = URLComponents()
        components.scheme = "devline"
        components.host = "docs"
        components.queryItems = [URLQueryItem(name: "id", value: document.id)]
        if let anchor = currentAnchor { components.queryItems?.append(URLQueryItem(name: "anchor", value: anchor)) }
        if let focus = currentFocus { components.queryItems?.append(URLQueryItem(name: "focus", value: focus)) }
        if let address = components.string { copyToPasteboard(address) }
    }
    private struct ReaderTarget: Equatable { let anchor, focus: String?; let position, offset: Double? }
}
