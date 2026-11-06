import SwiftUI

struct DocsView: View {
    @Environment(\.defaultMinListRowHeight) private var standardRowHeight
    @ObservedObject var model: WorkspaceModel
    @Binding var query: String
    @Binding var stack: String?
    @Binding var directDocumentID: String?
    let projectContext: ProjectContext?
    let clearProject: () -> Void
    let focusRequest: Int
    let openLibrary: () -> Void

    @State private var packReference: String?
    @State private var hits: [SearchHit] = []
    @State private var selectedID: String?
    @State private var committedSearch: SearchKey?
    @State private var reveal: ListReveal?
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
        SearchKey(query: trimmedQuery, stack: stack, pack: packReference, revision: model.libraryRevision)
    }
    private var loadingMore: Bool { pageRequest != nil }
    private var readingID: String? { directDocumentID ?? selectedID }
    private var readKey: ReadKey {
        ReadKey(id: readingID, query: directDocumentID == nil ? committedSearch?.query ?? "" : "")
    }

    var body: some View {
        VStack(spacing: 0) {
            searchBar
            if let projectContext { projectBar(projectContext) }
            Divider()
            content
        }
        .task(id: focusRequest) { searchFocused = true }
        .task(id: searchKey) { await search(searchKey) }
        .task(id: readKey) { await loadDocument(readKey) }
        .onChange(of: query) { _, _ in directDocumentID = nil }
        .onChange(of: stack) { _, stack in
            if let currentPack, currentPack.stack != stack { packReference = nil }
        }
        .onChange(of: model.libraryRevision) { _, _ in
            if let stack, !stacks.contains(stack) { self.stack = nil }
            if currentPack == nil { packReference = nil }
        }
    }

    @ViewBuilder private var content: some View {
        if directDocumentID != nil {
            reader
        } else if model.catalog == nil {
            ContentUnavailableView {
                Label("Local Library", systemImage: "tray")
            } description: {
                Text("Load the catalog to browse available documentation.")
            } actions: {
                Button("Reload Catalog") { Task { await model.refresh() } }
            }
        } else if packs.isEmpty {
            ContentUnavailableView {
                Label("Install Documentation", systemImage: "book.closed")
            } description: {
                Text("Choose the languages and frameworks you want to read offline.")
            } actions: {
                Button("Open Library", action: openLibrary).buttonStyle(.borderedProminent)
            }
        } else if trimmedQuery.isEmpty && currentPack == nil {
            libraryOverview
        } else if let searchError {
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

    private var noResults: some View {
        ContentUnavailableView {
            Label(isBrowsingPack ? "No Articles" : "No Results", systemImage: "magnifyingglass")
        } description: {
            if isBrowsingPack {
                Text("This release has no indexed articles.")
            } else if let currentPack {
                Text("No matches for “\(trimmedQuery)” in \(currentPack.name).")
            } else {
                Text("No matches for “\(trimmedQuery)” in the current scope.")
            }
        } actions: {
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
            } else if let detected = project.stack, stack == detected && currentPack == nil {
                Text("Project scope · \(displayStack(detected))").font(.caption).foregroundStyle(.secondary)
            } else {
                Text(project.stacks.map(displayStack).joined(separator: ", "))
                    .font(.caption).foregroundStyle(.secondary)
                    .lineLimit(1)
            }
            Spacer()
            if let stack = project.stack, !stacks.contains(stack) {
                Button("Install \(displayStack(stack)) Docs", action: openLibrary)
                    .controlSize(.small)
            } else if let detected = project.stack, stack != detected || currentPack != nil {
                Button("Use Project Scope") { setStack(detected) }
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
            if directDocumentID != nil || currentPack != nil {
                Button {
                    if directDocumentID != nil { directDocumentID = nil }
                    else { clearScope() }
                    searchFocused = true
                } label: {
                    Image(systemName: "chevron.left").frame(width: 17)
                }
                .help(currentPack == nil ? "Back to Search" : "Back to All Docsets")
                .accessibilityLabel(currentPack == nil ? "Back to Search" : "Back to All Docsets")
            }
            HStack(spacing: 8) {
                Image(systemName: "magnifyingglass").foregroundStyle(.secondary)
                TextField(currentPack.map { "Search \($0.name)" } ?? "Search symbols, APIs, or articles", text: $query)
                    .textFieldStyle(.plain)
                    .focused($searchFocused)
                    .onSubmit {
                        if selectedID == nil, let first = hits.first {
                            selectAndReveal(first.id)
                        }
                    }
                    .onKeyPress(.downArrow) { moveSelection(by: 1) }
                    .onKeyPress(.upArrow) { moveSelection(by: -1) }
                if !query.isEmpty {
                    Button {
                        query = ""
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
                Label("\(currentPack.name) · \(currentPack.version)", systemImage: "book")
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

    private func groupPacks(_ name: String) -> [DocPack] {
        packs.filter { $0.stack == name }
    }

    private var searchResults: some View {
        HSplitView {
            VStack(alignment: .leading, spacing: 0) {
                HStack {
                    if isBrowsingPack {
                        Text("Articles").font(.caption.weight(.semibold))
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
        List(selection: $selectedID) {
            ForEach(hits) { hit in
                let name = packName(hit.packId, stack: hit.stack)
                let title = ReaderMarkup.title(hit.title, fallback: "\(name) Overview")
                VStack(alignment: .leading, spacing: 4) {
                    Text(title).font(.body.weight(.medium))
                        .lineLimit(2, reservesSpace: isBrowsingPack)
                    if let match = hit.matchText, !match.isEmpty, title.caseInsensitiveCompare(match) != .orderedSame {
                        Text(ReaderMarkup.summary(match)).font(.caption.monospaced()).lineLimit(1)
                    }
                    if !isBrowsingPack {
                        Text("\(name) · \(hit.version)")
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
            DocReader(document: document, packName: packName(document.packId, stack: document.stack))
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
        if clearSearch { query = "" }
        stack = pack.stack
        packReference = pack.reference
    }

    private func setStack(_ value: String?) {
        directDocumentID = nil
        packReference = nil
        stack = value
    }

    private func clearScope() { setStack(nil) }

    private func resetToBrowse() {
        query = ""
        clearScope()
    }

    private func selectAndReveal(_ id: String) {
        selectedID = id
        reveal = ListReveal(articleID: id)
    }

    private func moveSelection(by offset: Int) -> KeyPress.Result {
        guard committedSearch == searchKey, !hits.isEmpty else { return .ignored }
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
            let results: [SearchHit] = try await Core.request(
                CoreRequest("docs.search", version: pack?.version, query: key.query,
                            stack: pack?.stack ?? key.stack, packId: pack?.id,
                            limit: key.query.isEmpty ? browseLimit + 1 : 30)
            )
            try Task.checkCancellation()
            guard key == searchKey else { return }
            hits = key.query.isEmpty ? Array(results.prefix(browseLimit)) : results
            moreArticles = key.query.isEmpty && results.count > browseLimit
            committedSearch = key
            if !hits.contains(where: { $0.id == selectedID }) {
                if let first = hits.first { selectAndReveal(first.id) }
                else { selectedID = nil }
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
                            packId: pack.id, limit: browseLimit + 1, offset: offset)
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

    private func loadDocument(_ key: ReadKey) async {
        document = nil
        documentError = nil
        guard let id = key.id else { return }
        do {
            let value: Document = try await Core.request(CoreRequest("docs.show", id: id, query: key.query))
            try Task.checkCancellation()
            document = value
        } catch is CancellationError {
        } catch {
            documentError = error.localizedDescription
        }
    }

    private struct ListReveal: Equatable {
        let articleID: String
        private let generation = UUID()
    }

    private struct SearchKey: Equatable {
        let query: String
        let stack, pack: String?
        let revision: Int
    }

    private struct ReadKey: Equatable {
        let id: String?
        let query: String
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
                Text(pack.version).font(.caption.monospacedDigit()).foregroundStyle(.secondary)
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
