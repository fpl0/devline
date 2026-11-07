import AppKit
import SwiftUI

struct DocsView: View {
    @Environment(\.defaultMinListRowHeight) private var standardRowHeight
    @ObservedObject var model: WorkspaceModel
    @ObservedObject var reading: ReadingSession
    @Binding var query: String
    @Binding var stack: String?
    @Binding var directDocumentID: String?
    let projectContext: ProjectContext?
    let updateProject: (ProjectContext) -> Void
    let clearProject: () -> Void
    let focusRequest: Int
    let lookupRevision: Int
    let openLibrary: () -> Void
    @AppStorage("devline.focusReading") private var focusReading = false

    @State private var showReferences = false
    @State private var showProjectReferences = false
    @State private var useProjectScope = true
    @State private var showSavedReferences = true
    @State private var externalLink: URL?
    @State private var navigationToken = UUID()
    @State private var navigationAnchor: String?
    @State private var navigationFocus: String?
    @State private var currentReaderAnchor: String?
    @State private var navigationPosition: Double?
    @State private var navigationOffset: Double?
    @State private var navigationQuery = ""
    @State private var restoringHistory = false
    @State private var automaticSelection = false
    @State private var browseCategory: String?
    @State private var packReference: String?
    @State private var hits: [SearchHit] = []
    @State private var selectedID: String?
    @State private var committedSearch: SearchKey?
    @State private var reveal: ListReveal?
    @State private var pendingBrowseReveal: BrowseReveal?
    @State private var document: Document?
    @State private var documentError: String?
    @State private var searchError: String?
    @State private var searching = false
    @State private var pageRequest: UUID?
    @State private var moreArticles = false
    @State private var browseError: String?
    private let browseLimit = 80
    private let browseRowHeight: CGFloat = 42
    private let browseRowInsets = EdgeInsets(top: 0, leading: 8, bottom: 0, trailing: 8)
    @FocusState private var searchFocused: Bool

    private var packs: [DocPack] { model.catalog?.docs.filter(\.installed) ?? [] }
    private var stacks: [String] { Array(Set(packs.map(\.stack))).sorted() }
    private var currentPack: DocPack? { packs.first { $0.reference == packReference } }
    private var currentPackVersions: [DocPack] {
        guard let currentPack else { return [] }
        return packs.filter { $0.id == currentPack.id }
            .sorted { $0.version.compare($1.version, options: .numeric) == .orderedDescending }
    }
    private var visibleStacks: [String] { stack.map { [$0] } ?? stacks }
    private var trimmedQuery: String { query.trimmingCharacters(in: .whitespacesAndNewlines) }
    private var isBrowsingPack: Bool { currentPack != nil && trimmedQuery.isEmpty }
    private var searchKey: SearchKey {
        SearchKey(query: trimmedQuery, stack: stack, pack: packReference, category: trimmedQuery.isEmpty ? browseCategory : nil,
                  revision: model.libraryRevision, project: useProjectScope ? projectContext?.path : nil,
                  pins: projectContext?.pins?.map { "\($0.id)@\($0.version)" }.joined(separator: ",") ?? "")
    }
    private var loadingMore: Bool { pageRequest != nil }
    private var readingID: String? { directDocumentID ?? selectedID }
    private var readKey: ReadKey {
        let symbol = hits.first(where: { $0.id == selectedID })?.symbol
        return ReadKey(id: readingID, query: directDocumentID == nil ? symbol ?? committedSearch?.query ?? "" : navigationQuery,
                       token: navigationToken)
    }
    private var browseRevealKey: BrowseRevealKey? {
        let scope = searchKey
        guard let target = pendingBrowseReveal, target.token == navigationToken,
              target.scope == scope, isBrowsingPack, !searching, committedSearch == scope else { return nil }
        return BrowseRevealKey(target: target, loaded: hits.count)
    }

    var body: some View {
        VStack(spacing: 0) {
            if !focusReading || readingID == nil {
                searchBar
                if let projectContext { projectBar(projectContext) }
                Divider()
            }
            content
        }
        .task(id: focusRequest) {
            if focusRequest > 0 { focusReading = false; searchFocused = true }
        }
        .onChange(of: focusReading) { _, _ in
            if let location = reading.history.current, location.id == readingID {
                navigationPosition = location.position
                navigationAnchor = location.anchor
                navigationOffset = location.anchorOffset
            }
        }
        .task(id: searchKey) { await search(searchKey) }
        .task(id: readKey) { await loadDocument(readKey) }
        .task(id: browseRevealKey) { if let key = browseRevealKey { await revealBrowseHistory(key) } }
        .onChange(of: searchKey) { _, scope in
            if let target = pendingBrowseReveal, target.scope != scope { pendingBrowseReveal = nil }
        }
        .task { if let request = reading.request { restore(request) }; await reading.refresh() }
        .onChange(of: reading.error) { _, error in if let error { model.error = error; reading.error = nil } }
        .onChange(of: reading.request) { _, request in if let request { restore(request) } }
        .sheet(isPresented: $showReferences) { ReferenceShelf(reading: reading, open: openReference, initialSaved: showSavedReferences) }
        .sheet(isPresented: $showProjectReferences) {
            if let projectContext { ProjectReferencesView(project: projectContext, updated: updateProject) }
        }
        .onChange(of: projectContext?.path) { _, _ in useProjectScope = true; packReference = nil; browseCategory = nil }
        .onChange(of: lookupRevision) { _, _ in
            useProjectScope = true; packReference = nil; browseCategory = nil; resetReaderTarget()
        }
        .alert("Link Unavailable Offline", isPresented: Binding(get: { externalLink != nil }, set: { if !$0 { externalLink = nil } })) {
            Button("Open in Browser") { if let url = externalLink { NSWorkspace.shared.open(url) }; externalLink = nil }
            Button("Cancel", role: .cancel) { externalLink = nil }
        } message: {
            Text("This page or section is unavailable in your installed documentation. Open \(externalLink?.host ?? "the linked page") in your browser when you have a connection.")
        }
        .onChange(of: stack) { _, stack in
            if let currentPack, currentPack.stack != stack { packReference = nil }
        }
        .onChange(of: model.libraryRevision) { _, _ in
            if let stack, !stacks.contains(stack) { self.stack = nil }
            if currentPack == nil { packReference = nil }
        }
    }

    @ViewBuilder private var content: some View {
        if focusReading && readingID != nil {
            reader
        } else if directDocumentID != nil && hits.isEmpty {
            reader
        } else if directDocumentID == nil && model.catalog == nil {
            ContentUnavailableView {
                Label("Local Library", systemImage: "tray")
            } description: {
                Text("Load the catalog to browse available documentation.")
            } actions: {
                Button("Reload Catalog") { Task { await model.refresh() } }
            }
        } else if directDocumentID == nil && packs.isEmpty {
            ContentUnavailableView {
                Label("Install Documentation", systemImage: "book.closed")
            } description: {
                Text("Choose the languages and frameworks you want to read offline.")
            } actions: {
                Button("Open Library", action: openLibrary).buttonStyle(.borderedProminent)
            }
        } else if directDocumentID == nil && trimmedQuery.isEmpty && currentPack == nil {
            libraryOverview
        } else if directDocumentID == nil, let searchError {
            ContentUnavailableView {
                Label("Documentation Unavailable", systemImage: "exclamationmark.circle")
            } description: {
                Text(searchError)
            } actions: {
                Button("Browse Docsets", action: resetToBrowse)
            }
        } else if hits.isEmpty && searching {
            ProgressView().frame(maxWidth: .infinity, maxHeight: .infinity)
        } else if hits.isEmpty {
            noResults
        } else {
            searchResults
        }
    }

    private var categoryTitle: String {
        switch browseCategory {
        case "guide": "Guides"
        case "api": "APIs"
        case "reference": "Reference"
        case "compatibility": "Compatibility Notes"
        default: "All Articles"
        }
    }

    private var noResults: some View {
        ContentUnavailableView {
            Label(isBrowsingPack ? "No Articles" : "No Results", systemImage: "magnifyingglass")
        } description: {
            if isBrowsingPack, browseCategory != nil {
                Text("This release has no articles in \(categoryTitle).")
            } else if isBrowsingPack {
                Text("This release has no indexed articles.")
            } else if let currentPack {
                Text("No matches for “\(trimmedQuery)” in \(currentPack.name).")
            } else {
                Text("No matches for “\(trimmedQuery)” in the current scope.")
            }
        } actions: {
            if isBrowsingPack, browseCategory != nil {
                Button("All Articles") { browseCategory = nil }
            }
            if packReference != nil || stack != nil {
                Button("Search All Docsets") { clearScope() }.buttonStyle(.borderedProminent)
            }
            Button("Browse Docsets", action: resetToBrowse)
        }
    }

    private func projectBar(_ project: ProjectContext) -> some View {
        HStack(spacing: 9) {
            Image(systemName: "folder").foregroundStyle(.secondary)
            Text(URL(fileURLWithPath: project.path).lastPathComponent)
                .font(.callout.weight(.medium))
                .lineLimit(1)
                .help(project.path)
            if project.stacks.isEmpty {
                Text("No supported stack detected").font(.caption).foregroundStyle(.secondary)
            } else if let detected = project.stack, stack == detected && currentPack == nil && useProjectScope {
                Text("Project scope · \(displayStack(detected))").font(.caption).foregroundStyle(.secondary)
            } else {
                Text(project.stacks.map(displayStack).joined(separator: ", "))
                    .font(.caption).foregroundStyle(.secondary)
                    .lineLimit(1)
            }
            Spacer()
            Button {
                showProjectReferences = true
            } label: {
                Label((project.docsets ?? []).contains(where: \.needsAttention) ? "Review Versions" : "Project References",
                      systemImage: (project.docsets ?? []).contains(where: \.needsAttention) ? "exclamationmark.triangle" : "pin")
            }.controlSize(.small)
            if let stack = project.stack, !stacks.contains(stack) {
                Button("Install \(displayStack(stack)) Docs", action: openLibrary)
                    .controlSize(.small)
            } else if let detected = project.stack, stack != detected || currentPack != nil || !useProjectScope {
                Button("Use Project Scope") { setStack(detected); useProjectScope = true }
                    .controlSize(.small)
            }
            Button(action: clearProject) { Image(systemName: "xmark.circle") }
                .buttonStyle(.borderless)
                .help("Clear Project Context")
                .accessibilityLabel("Clear Project Context")
        }
        .padding(.horizontal, 18)
        .padding(.bottom, 10)
    }

    private var searchBar: some View {
        HStack(spacing: 10) {
            HStack(spacing: 3) {
                Button { reading.move(-1) } label: { Image(systemName: "chevron.left").frame(width: 16) }
                    .disabled(!reading.history.canGoBack).keyboardShortcut("[")
                    .help("Back (⌘[)").accessibilityLabel("Back")
                Button { reading.move(1) } label: { Image(systemName: "chevron.right").frame(width: 16) }
                    .disabled(!reading.history.canGoForward).keyboardShortcut("]")
                    .help("Forward (⌘])").accessibilityLabel("Forward")
                Button(action: resetToBrowse) { Image(systemName: "books.vertical").frame(width: 16) }
                    .help("Browse Documentation").accessibilityLabel("Browse Documentation")
            }.buttonStyle(.borderless)
            Button { showSavedReferences = true; showReferences = true } label: { Image(systemName: "bookmark").frame(width: 16) }
                .buttonStyle(.borderless).keyboardShortcut("b", modifiers: [.command, .shift])
                .help("Saved and Recent References (⇧⌘B)").accessibilityLabel("Saved and Recent References")
            HStack(spacing: 8) {
                Image(systemName: "magnifyingglass").foregroundStyle(.secondary)
                TextField(currentPack.map { "Search \($0.name)" } ?? "Search docs — try py:Path.read_text", text: Binding(
                    get: { query }, set: { query = $0; directDocumentID = nil; resetReaderTarget() }
                ))
                    .textFieldStyle(.plain)
                    .focused($searchFocused)
                    .onSubmit(commitSearchSelection)
                    .onKeyPress(.downArrow) { moveSelection(by: 1) }
                    .onKeyPress(.upArrow) { moveSelection(by: -1) }
                if !query.isEmpty {
                    Button {
                        query = ""
                        directDocumentID = nil
                        resetReaderTarget()
                        searchFocused = true
                    } label: {
                        Image(systemName: "xmark.circle.fill").foregroundStyle(.secondary)
                    }
                    .buttonStyle(.borderless)
                    .accessibilityLabel("Clear search")
                } else {
                    Text("⌘F").font(.caption).foregroundStyle(.tertiary)
                }
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 8)
            .background(.quaternary, in: RoundedRectangle(cornerRadius: 7))
            scopeMenu
        }
        .padding(.horizontal, 18)
        .padding(.vertical, 12)
    }

    private var scopeMenu: some View {
        Menu {
            Button("All Docsets") { clearScope() }
            if currentPackVersions.count > 1 {
                Section("Releases") {
                    ForEach(currentPackVersions, id: \.reference) { pack in
                        Button { choosePack(pack, clearSearch: false) } label: {
                            if pack.reference == packReference { Label(pack.version, systemImage: "checkmark") }
                            else { Text(pack.version) }
                        }
                    }
                }
            }
            Section("Stacks") {
                ForEach(stacks, id: \.self) { name in
                    Menu(displayStack(name)) {
                        Button("All \(displayStack(name)) Docsets") { setStack(name) }
                        Divider()
                        ForEach(groupPacks(name), id: \.reference) { pack in
                            Button("\(pack.name) · \(pack.version)") { choosePack(pack, clearSearch: false) }
                        }
                    }
                }
            }
        } label: {
            if let currentPack {
                Label("\(currentPack.name) · \(currentPack.version.components(separatedBy: "+")[0])", systemImage: "book")
                    .lineLimit(1)
            } else {
                Text(stack.map(displayStack) ?? "All Docsets").lineLimit(1)
            }
        }
        .menuStyle(.borderedButton)
        .fixedSize()
        .accessibilityLabel("Documentation scope")
    }

    private var libraryOverview: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 22) {
                HStack(alignment: .firstTextBaseline) {
                    VStack(alignment: .leading, spacing: 6) {
                        Text("Browse Documentation")
                            .font(.title2.weight(.semibold))
                            .accessibilityAddTraits(.isHeader)
                        Text("Open a docset or search your offline library.")
                            .font(.callout).foregroundStyle(.secondary)
                    }
                    Spacer()
                    Button("Manage Library", action: openLibrary)
                        .controlSize(.small)
                }

                if !reading.library.saved.isEmpty || !reading.library.recent.isEmpty {
                    referenceOverview
                }
                HStack(spacing: 8) {
                    Text("Quick lookup").font(.caption).foregroundStyle(.secondary)
                    ForEach(["py:Path.read_text", "go:http.Handler", "rust:Vec::push"], id: \.self) { example in
                        Button(example) { clearScope(); query = example; searchFocused = true }
                            .buttonStyle(.link).font(.caption.monospaced())
                    }
                }
                LazyVGrid(columns: [GridItem(.adaptive(minimum: 220, maximum: 335), spacing: 26, alignment: .topLeading)],
                          alignment: .leading, spacing: 22) {
                    ForEach(visibleStacks, id: \.self) { name in
                        VStack(alignment: .leading, spacing: 7) {
                            Text(displayStack(name))
                                .font(.caption.weight(.semibold))
                                .foregroundStyle(.secondary)
                                .accessibilityAddTraits(.isHeader)
                            Divider()
                            ForEach(groupPacks(name), id: \.reference) { pack in
                                DocsetRow(pack: pack) { choosePack(pack, clearSearch: true) }
                            }
                        }
                    }
                }
            }
            .frame(maxWidth: 950, alignment: .leading)
            .padding(.horizontal, 26)
            .padding(.vertical, 24)
            .frame(maxWidth: .infinity, alignment: .topLeading)
        }
    }

    private var referenceOverview: some View {
        HStack(alignment: .top, spacing: 32) {
            referenceColumn("Saved", icon: "bookmark", entries: Array(reading.library.saved.prefix(4)))
            referenceColumn("Recently Opened", icon: "clock", entries: Array(reading.library.recent.prefix(4)))
        }
    }

    private func referenceColumn(_ title: String, icon: String, entries: [ReadingReference]) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Label(title, systemImage: icon).font(.callout.weight(.semibold))
                Spacer()
                Button("View All") { showSavedReferences = icon == "bookmark"; showReferences = true }.font(.caption).buttonStyle(.link)
            }
            if entries.isEmpty {
                Text("Keep useful pages here with ⌘D.").font(.caption).foregroundStyle(.secondary)
            }
            ForEach(entries) { reference in
                Button { openReference(reference) } label: {
                    VStack(alignment: .leading, spacing: 3) {
                        Text(ReaderMarkup.title(reference.title, fallback: reference.packId)).lineLimit(1)
                        Text("\(reference.packId) · \(displayDocVersion(reference.version))")
                            .font(.caption).foregroundStyle(.secondary)
                    }.frame(maxWidth: .infinity, alignment: .leading).contentShape(Rectangle())
                }.buttonStyle(.plain).disabled(!reference.installed).padding(.vertical, 3)
            }
        }.frame(maxWidth: .infinity, alignment: .topLeading)
    }

    private func groupPacks(_ name: String) -> [DocPack] {
        Dictionary(grouping: packs.filter { $0.stack == name }, by: \.id).values
            .compactMap { versions in versions.first { $0.active == true } ?? versions.max { $0.version.compare($1.version, options: .numeric) == .orderedAscending } }
            .sorted { $0.name.localizedStandardCompare($1.name) == .orderedAscending }
    }

    private var searchResults: some View {
        HSplitView {
            VStack(alignment: .leading, spacing: 0) {
                HStack {
                    if isBrowsingPack {
                        Menu {
                            Button("All Articles") { browseCategory = nil }
                            Divider()
                            Button("Guides") { browseCategory = "guide" }
                            Button("APIs") { browseCategory = "api" }
                            Button("Reference") { browseCategory = "reference" }
                            Button("Compatibility Notes") { browseCategory = "compatibility" }
                        } label: {
                            Text(categoryTitle).font(.caption.weight(.semibold))
                        }.menuStyle(.borderlessButton).fixedSize().accessibilityLabel("Article Category")
                        Spacer()
                        Text("\(hits.count) loaded").font(.caption.monospacedDigit())
                    } else {
                        Text("\(hits.count) \(hits.count == 1 ? "result" : "results")").font(.caption.weight(.semibold))
                        Spacer()
                        if searching { ProgressView().controlSize(.mini) }
                    }
                }
                .foregroundStyle(.secondary)
                .padding(.horizontal, 13)
                .padding(.vertical, 9)
                ScrollViewReader { proxy in
                    articleList
                        .onChange(of: reveal) { _, request in
                            if let request { proxy.scrollTo(request.articleID) }
                        }
                }
            }
            .frame(minWidth: 195, idealWidth: 245, maxWidth: 290)
            .background(.quaternary.opacity(0.22))
            reader
                .frame(minWidth: 345, maxWidth: .infinity, maxHeight: .infinity)
        }
    }

    @ViewBuilder private var articleList: some View {
        if #available(macOS 15, *) {
            articleListContent
                .onScrollGeometryChange(for: Bool.self) { geometry in
                    geometry.contentOffset.y > 0 && geometry.visibleRect.height > 0
                        && geometry.visibleRect.maxY + 120 >= geometry.contentSize.height
                } action: { _, nearEnd in
                    if nearEnd && browseError == nil {
                        Task { await loadMoreArticles() }
                    }
                }
        } else {
            articleListContent
        }
    }

    private var articleListContent: some View {
        List(selection: Binding(get: { selectedID }, set: { value in
            guard value != selectedID else { return }
            if value != nil { directDocumentID = nil; resetReaderTarget(); automaticSelection = false }
            selectedID = value
        })) {
            ForEach(hits) { hit in
                let name = packName(hit.packId, stack: hit.stack)
                let title = ReaderMarkup.title(hit.title, fallback: "\(name) Overview")
                VStack(alignment: .leading, spacing: 4) {
                    Text(!isBrowsingPack ? hit.symbol ?? title : title).font(.body.weight(.medium))
                        .lineLimit(2, reservesSpace: isBrowsingPack)
                    if !isBrowsingPack, hit.symbol != nil {
                        Text(title).font(.caption).foregroundStyle(.secondary).lineLimit(1)
                    }
                    if let match = hit.matchText, !match.isEmpty, hit.symbol == nil, title.caseInsensitiveCompare(match) != .orderedSame {
                        Text(ReaderMarkup.summary(match)).font(.caption.monospaced()).lineLimit(1)
                    }
                    if !isBrowsingPack {
                        Text("\(name) · \(displayDocVersion(hit.version))")
                            .font(.caption).foregroundStyle(.secondary).lineLimit(1)
                        if hit.matchText == nil && !hit.snippet.isEmpty {
                            Text(ReaderMarkup.summary(hit.snippet))
                                .font(.caption).foregroundStyle(.secondary).lineLimit(2)
                        }
                    }
                }
                .padding(.vertical, isBrowsingPack ? 0 : 5)
                .frame(height: isBrowsingPack ? browseRowHeight : nil, alignment: .leading)
                .listRowInsets(isBrowsingPack ? browseRowInsets : nil)
                .tag(hit.id)
                .id(hit.id)
            }
            if isBrowsingPack {
                paginationRow
                    .frame(minHeight: browseRowHeight, alignment: .leading)
                    .listRowInsets(browseRowInsets)
                    .selectionDisabled()
                    .listRowSeparator(.hidden)
            }
        }
        .environment(\.defaultMinListRowHeight, isBrowsingPack ? browseRowHeight : standardRowHeight)
        .listStyle(.inset)
        .scrollContentBackground(.hidden)
    }

    private var paginationRow: some View {
        VStack(alignment: .leading, spacing: 5) {
            if searching || loadingMore {
                HStack(spacing: 7) {
                    ProgressView().controlSize(.mini)
                    Text("Loading Articles…").font(.callout).foregroundStyle(.secondary)
                }
            } else if moreArticles {
                Button(browseError == nil ? "Load More Articles" : "Retry Loading Articles") {
                    Task { await loadMoreArticles() }
                }
                .buttonStyle(.borderless)
            } else {
                Label("All Articles Loaded", systemImage: "checkmark.circle")
                    .font(.caption).foregroundStyle(.secondary)
            }
            if let browseError {
                Text(browseError).font(.caption).foregroundStyle(.secondary)
            }
        }
        .padding(.vertical, 5)
    }

    @ViewBuilder private var reader: some View {
        if let document, document.id == readingID {
            DocReader(document: document, packName: packName(document.packId, stack: document.stack),
                      anchor: navigationAnchor, focus: navigationFocus, restoredPosition: navigationPosition, restoredOffset: navigationOffset,
                      isSaved: reading.isSaved(document.id),
                      save: { anchor, focus in
                          var location = currentLocation(document)
                          location.anchor = anchor
                          location.focusText = focus
                          Task { await reading.toggleSaved(location) }
                      },
                      navigate: { url in Task { await follow(url, from: document) } },
                      openPart: { id in reading.open(ReferenceLocation(id: id, query: query, stack: stack, pack: packReference, category: browseCategory)) },
                      reportPosition: { [token = navigationToken] fraction, anchor, offset in
                          guard token == navigationToken else { return }
                          currentReaderAnchor = anchor
                          reading.updatePosition(id: document.id, fraction: fraction, anchor: anchor, anchorOffset: offset)
                      })
                .id(document.id)
        } else if let documentError {
            ContentUnavailableView("Document Unavailable", systemImage: "exclamationmark.circle", description: Text(documentError))
        } else if readingID != nil {
            ProgressView().frame(maxWidth: .infinity, maxHeight: .infinity)
        } else {
            ContentUnavailableView("Select an Article", systemImage: "doc.text")
        }
    }

    private func packName(_ id: String, stack: String) -> String {
        packs.first { $0.id == id }?.name ?? displayStack(stack)
    }

    private func choosePack(_ pack: DocPack, clearSearch: Bool) {
        directDocumentID = nil
        resetReaderTarget()
        browseCategory = nil
        if clearSearch { query = "" }
        stack = pack.stack
        packReference = pack.reference
    }

    private func setStack(_ value: String?) {
        useProjectScope = false
        directDocumentID = nil
        packReference = nil
        browseCategory = nil
        stack = value
    }

    private func clearScope() { setStack(nil) }

    private func resetToBrowse() {
        query = ""
        clearScope()
    }

    private func selectAndReveal(_ id: String) {
        directDocumentID = nil
        resetReaderTarget()
        selectedID = id
        reveal = ListReveal(articleID: id)
    }

    private func commitSearchSelection() {
        guard committedSearch == searchKey else { return }
        let selection = hits.first { $0.id == selectedID } ?? hits.first
        guard let selection else { return }
        automaticSelection = false
        if let document, document.id == selection.id {
            let location = currentLocation(document)
            Task { await reading.didOpen(location, restoring: false, searchPreview: false) }
        } else if selectedID != selection.id || directDocumentID != nil {
            selectAndReveal(selection.id)
        }
    }

    private func moveSelection(by offset: Int) -> KeyPress.Result {
        guard committedSearch == searchKey, !hits.isEmpty else { return .ignored }
        automaticSelection = false
        let index = hits.firstIndex { $0.id == selectedID } ?? -1
        if index + offset >= hits.count, isBrowsingPack, moreArticles {
            Task { await loadMoreArticles(advanceSelection: true) }
        } else {
            let next = hits[min(max(index + offset, 0), hits.count - 1)].id
            if next != selectedID { selectAndReveal(next) }
        }
        return .handled
    }

    private func search(_ key: SearchKey) async {
        let pack = currentPack
        pageRequest = nil
        browseError = nil
        moreArticles = false
        guard (!key.query.isEmpty || pack != nil), !packs.isEmpty else {
            hits = []
            selectedID = nil
            committedSearch = nil
            searchError = nil
            searching = false
            return
        }

        searchError = nil
        searching = true
        do {
            if !key.query.isEmpty { try await Task.sleep(for: .milliseconds(30)) }
            let inheritedScope = pack == nil && key.project != nil
            let results: [SearchHit] = try await Core.request(
                CoreRequest("docs.search", version: pack?.version, query: key.query,
                            stack: inheritedScope ? nil : pack?.stack ?? key.stack, packId: pack?.id,
                            limit: key.query.isEmpty ? browseLimit + 1 : 30,
                            category: key.category, projectStack: inheritedScope ? key.stack : nil,
                            projectPath: inheritedScope ? key.project : nil)
            )
            try Task.checkCancellation()
            guard key == searchKey else { return }
            hits = key.query.isEmpty ? Array(results.prefix(browseLimit)) : results
            moreArticles = key.query.isEmpty && results.count > browseLimit
            committedSearch = key
            if directDocumentID == nil { automaticSelection = true }
            if directDocumentID == nil, !hits.contains(where: { $0.id == selectedID }) {
                if let first = hits.first { selectAndReveal(first.id) }
                else { selectedID = nil }
            }
            if pendingBrowseReveal == nil, let id = directDocumentID, hits.contains(where: { $0.id == id }) {
                selectedID = id
                reveal = ListReveal(articleID: id)
            }
            searching = false
        } catch is CancellationError {
        } catch {
            guard key == searchKey else { return }
            searchError = error.localizedDescription
            hits = []
            selectedID = nil
            committedSearch = nil
            searching = false
        }
    }

    private func loadMoreArticles(advanceSelection: Bool = false) async {
        guard let pack = currentPack, isBrowsingPack, moreArticles, !loadingMore,
              !searching, committedSearch == searchKey else { return }
        let key = searchKey
        let request = UUID()
        let offset = hits.count
        let previousSelection = selectedID
        pageRequest = request
        browseError = nil
        defer {
            if pageRequest == request { pageRequest = nil }
        }
        do {
            let next: [SearchHit] = try await Core.request(
                CoreRequest("docs.search", version: pack.version, query: "", stack: pack.stack,
                            packId: pack.id, limit: browseLimit + 1, offset: offset, category: key.category)
            )
            guard pageRequest == request, key == searchKey, hits.count == offset else { return }
            hits.append(contentsOf: next.prefix(browseLimit))
            moreArticles = next.count > browseLimit
            if advanceSelection, selectedID == previousSelection, let first = next.first {
                selectAndReveal(first.id)
            }
        } catch is CancellationError {
        } catch {
            guard pageRequest == request, key == searchKey else { return }
            browseError = error.localizedDescription
        }
    }

    private func revealBrowseHistory(_ key: BrowseRevealKey) async {
        guard key == browseRevealKey, !Task.isCancelled else { return }
        if hits.contains(where: { $0.id == key.target.id }) {
            selectedID = key.target.id
            reveal = ListReveal(articleID: key.target.id)
            pendingBrowseReveal = nil
        } else if moreArticles, hits.count <= key.target.index {
            await loadMoreArticles()
        } else {
            pendingBrowseReveal = nil
        }
    }

    private func loadDocument(_ key: ReadKey) async {
        guard directDocumentID != nil || committedSearch == searchKey else { return }
        documentError = nil
        if document?.id != key.id { document = nil }
        guard let id = key.id else { return }
        do {
            let value: Document = try await Core.request(CoreRequest("docs.show", id: id, query: key.query))
            try Task.checkCancellation()
            guard key == readKey else { return }
            document = value
            let restoring = restoringHistory
            restoringHistory = false
            await reading.didOpen(currentLocation(value), restoring: restoring, searchPreview: automaticSelection)
        } catch is CancellationError {
        } catch {
            guard key == readKey else { return }
            documentError = error.localizedDescription
        }
    }

    private func resetReaderTarget() {
        navigationAnchor = nil
        currentReaderAnchor = nil
        navigationFocus = nil
        navigationPosition = nil
        navigationOffset = nil
        navigationQuery = ""
        restoringHistory = false
        pendingBrowseReveal = nil
        navigationToken = UUID()
    }

    private func currentLocation(_ value: Document) -> ReferenceLocation {
        ReferenceLocation(id: value.id, query: query, stack: stack, pack: packReference, category: browseCategory,
                          anchor: currentReaderAnchor ?? navigationAnchor,
                          focusText: currentReaderAnchor == nil ? navigationFocus ?? value.focusText : nil,
                          browseIndex: isBrowsingPack ? hits.firstIndex { $0.id == value.id } : nil)
    }

    private func restore(_ request: ReferenceRequest) {
        let location = request.location
        query = location.query
        stack = location.stack
        packReference = location.pack
        browseCategory = location.category
        selectedID = location.id
        directDocumentID = location.id
        navigationQuery = location.query
        navigationAnchor = location.anchor
        currentReaderAnchor = location.anchor
        navigationFocus = location.focusText
        navigationPosition = location.position
        navigationOffset = location.anchorOffset
        navigationToken = request.token
        restoringHistory = request.restoring
        automaticSelection = false
        if request.restoring, location.pack != nil, trimmedQuery.isEmpty, let index = location.browseIndex {
            let scope = SearchKey(query: "", stack: location.stack, pack: location.pack,
                                  category: location.category, revision: model.libraryRevision, project: searchKey.project, pins: searchKey.pins)
            pendingBrowseReveal = BrowseReveal(id: location.id, index: index, token: request.token, scope: scope)
        } else {
            pendingBrowseReveal = nil
        }
        if reading.request?.token == request.token { reading.request = nil }
    }

    private func openReference(_ reference: ReadingReference) {
        reading.open(ReferenceLocation(id: reference.id, stack: reference.stack,
                                       pack: "\(reference.packId)@\(reference.version)",
                                       anchor: reference.anchor, focusText: reference.focusText))
    }

    private func follow(_ url: URL, from document: Document) async {
        let origin = readKey
        do {
            let resolved: ResolvedReference? = try await Core.request(CoreRequest("docs.resolve", id: document.id,
                                                                                version: document.version,
                                                                                packId: document.packId, url: url.absoluteString))
            guard origin == readKey else { return }
            if let resolved {
                reading.open(ReferenceLocation(id: resolved.id, query: query, stack: stack, pack: packReference, category: browseCategory,
                                               anchor: resolved.anchor, focusText: resolved.focusText))
            } else { externalLink = url }
        } catch { if origin == readKey { model.error = error.localizedDescription } }
    }

    private struct ListReveal: Equatable {
        let articleID: String
        private let generation = UUID()
    }

    private struct BrowseReveal: Equatable {
        let id: String
        let index: Int
        let token: UUID
        let scope: SearchKey
    }

    private struct BrowseRevealKey: Equatable {
        let target: BrowseReveal
        let loaded: Int
    }

    private struct SearchKey: Equatable {
        let query: String
        let stack, pack: String?
        let category: String?
        let revision: Int
        let project: String?
        let pins: String
    }

    private struct ReadKey: Equatable {
        let id: String?
        let query: String
        let token: UUID
    }
}

private struct DocsetRow: View {
    let pack: DocPack
    let select: () -> Void
    @State private var hovered = false

    var body: some View {
        Button(action: select) {
            HStack(spacing: 7) {
                Image(systemName: "book.closed").foregroundStyle(.secondary)
                Text(pack.name).font(.callout.weight(.medium)).lineLimit(1)
                Spacer(minLength: 6)
                Text(pack.version.components(separatedBy: "+")[0]).font(.caption.monospacedDigit()).foregroundStyle(.secondary)
                Image(systemName: "chevron.right").font(.caption2.weight(.semibold)).foregroundStyle(.tertiary)
            }
            .padding(.horizontal, 7)
            .padding(.vertical, 7)
            .background {
                if hovered { RoundedRectangle(cornerRadius: 5).fill(.quaternary) }
            }
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .onHover { hovered = $0 }
        .help(pack.description)
        .accessibilityLabel("Browse \(pack.name) \(pack.version)")
    }
}
