import AppKit
import SwiftUI

struct DocsView: View {
    @ObservedObject var model: WorkspaceModel
    @ObservedObject var reading: ReadingSession
    @Binding var query: String
    @Binding var stack: String?
    @Binding var packReference: String?
    @Binding var directDocumentID: String?
    let projectContext: ProjectContext?
    let updateProject: (ProjectContext) -> Void
    let clearProject: () -> Void
    let focusRequest: Int
    let lookupRevision: Int
    let projectScopeRequested: Bool
    let isActive: Bool
    let openLibrary: () -> Void
    let installDocset: (String?) -> Void

    @Preference("devline.focusReading") private var focusReading = false
    @FocusState private var searchFocused: Bool
    @State private var scope: Scope = .installed
    @State private var group: String?
    @State private var browsedPackInProject = false
    @State private var navigation: (key: NavigationKey, value: DocsetNavigation)?
    @State private var showReferences = false
    @State private var showProjectReferences = false
    @State private var openedLocation: ReferenceLocation?
    @State private var navigationToken = UUID()
    @State private var currentReaderAnchor: String?
    @State private var restoring = false
    @State private var loadedDocument: LoadedDocument?
    @State private var recordedRead: ReadKey?
    @State private var documentFailure: (key: ReadKey, message: String)?
    @State private var linkIssue: LinkIssue?

    @State private var hits: [SearchHit] = []
    @State private var selectedID: String?
    @State private var committedSearch: SearchKey?
    @State private var submittedSearch: SearchKey?
    @State private var searchFailure: (key: SearchKey, message: String)?
    @State private var pageRequest: UUID?
    @State private var moreArticles = false
    @State private var browseError: String?
    @State private var reveal: UUID?
    @State private var pendingBrowseReveal: (id: String, index: Int)?
    private let browseLimit = 80
    private let searchLimit = 30

    private var packs: [DocPack] { model.catalog?.docs.filter(\.installed) ?? [] }
    private var selectedPack: DocPack? { packs.first { $0.reference == packReference } }
    private var currentPack: DocPack? {
        if scope != .docset, let document, let pack = docset(document.packId, version: document.version) { return pack }
        return selectedPack
    }
    private var scopedPack: DocPack? { scope == .docset ? currentPack : nil }
    private var trimmedQuery: String { query.trimmingCharacters(in: .whitespacesAndNewlines) }
    private var browsing: Bool { trimmedQuery.isEmpty && scopedPack != nil }
    private var hasCurrentResults: Bool { committedSearch == searchKey }
    private var searchError: String? { searchFailure?.key == searchKey ? searchFailure?.message : nil }
    private var searching: Bool { !hasCurrentResults && searchError == nil }
    private var projectPins: String {
        projectContext?.pins?.map { "\($0.id)@\($0.version)" }.joined(separator: ",") ?? ""
    }
    private var projectLinkPath: String? { openedLocation?.projectScoped == true ? projectContext?.path : nil }
    private var searchKey: SearchKey {
        let term = trimmedQuery
        let pack = scopedPack?.reference
        let isBrowsing = term.isEmpty && pack != nil
        return SearchKey(query: term, scope: scope, pack: pack, group: isBrowsing ? group : nil,
                         stack: scope == .installed ? stack : nil,
                         category: isBrowsing && group == nil ? openedLocation?.category : nil,
                         revision: model.libraryRevision, project: scope == .project ? projectContext?.path : nil,
                         pins: scope == .project ? projectPins : "")
    }
    private var readKey: ReadKey { ReadKey(id: directDocumentID, token: navigationToken) }
    private var document: Document? { loadedDocument?.key == readKey ? loadedDocument?.value : nil }
    private var visiblePacks: [DocPack] {
        if scope == .project {
            let releases = Set(projectContext?.docsets?.compactMap { docset in
                docset.version.map { "\(docset.id)@\($0)" }
            } ?? [])
            return packs.filter { releases.contains($0.reference) }.sorted(by: packPrecedes)
        }
        let matching = packs.filter { isImported($0.id) == (scope == .imported) }
        return Dictionary(grouping: matching, by: \.id).values
            .compactMap { versions in versions.first(where: { $0.active == true }) ?? versions.first }
            .sorted(by: packPrecedes)
    }

    var body: some View {
        let key = searchKey
        let target = readKey
        VStack(spacing: 0) {
            if focusReading && directDocumentID != nil {
                reader
            } else {
                scopeBar
                Divider()
                HSplitView {
                    sidebar(key).frame(minWidth: 190, idealWidth: 225, maxWidth: 275, maxHeight: .infinity)
                    reader.frame(minWidth: 345, maxWidth: .infinity, maxHeight: .infinity)
                }
            }
        }
        .toolbar { windowToolbar }
        .task {
            if let request = reading.request { restore(request) }
            else { applyEntryPoint() }
        }
        .onChange(of: focusRequest, initial: true) { _, request in
            if isActive && request > 0 { focusReading = false; searchFocused = true }
        }
        .onChange(of: isActive) { _, active in
            if !active {
                searchFocused = false
                showReferences = false
                showProjectReferences = false
            }
        }
        .task(id: isActive) {
            if isActive { await reading.refresh() }
        }
        .task(id: isActive ? key : nil) {
            if isActive && committedSearch != key { await search(key) }
        }
        .task(id: isActive ? target : nil) {
            if isActive && recordedRead != target { await loadDocument(target) }
        }
        .task(id: isActive ? hits.count : nil) {
            if isActive { await revealBrowseHistory() }
        }
        .onChange(of: lookupRevision) { _, _ in applyEntryPoint() }
        .onChange(of: reading.request) { _, request in if let request { restore(request) } }
        .onChange(of: reading.error) { _, error in if let error { model.error = error; reading.error = nil } }
        .onChange(of: projectContext?.path) { _, path in
            if path == nil && (scope == .project || (scope == .docset && scopedPack == nil)) {
                changeScope(.installed)
            }
        }
        .onChange(of: model.libraryRevision) { _, _ in
            if scope == .docset && currentPack == nil { changeScope(.installed) }
            if let document, docset(document.packId, version: document.version) == nil {
                navigationToken = UUID()
            }
            linkIssue = nil
        }
        .onChange(of: focusReading) { _, _ in
            guard let location = reading.history.current, location.id == directDocumentID else { return }
            openedLocation?.position = location.position
            openedLocation?.anchor = location.anchor
            openedLocation?.anchorOffset = location.anchorOffset
        }
        .sheet(isPresented: $showReferences) { ReferenceShelf(reading: reading, open: openReference) }
        .sheet(isPresented: $showProjectReferences) {
            if let projectContext { ProjectReferencesView(project: projectContext, model: model, updated: updateProject) }
        }
    }

    /// Navigation and search live in the window toolbar, like a browser's.
    @ToolbarContentBuilder private var windowToolbar: some ToolbarContent {
        // Made once and hidden while another page is on screen: adding and removing toolbar items
        // is the slowest part of changing page.
        if #available(macOS 15.0, *) {
            ToolbarItemGroup(placement: .navigation) { historyButtons }.hidden(!isActive)
            ToolbarItem(placement: .principal) { searchField }.hidden(!isActive)
            ToolbarItemGroup(placement: .primaryAction) { libraryButtons }.hidden(!isActive)
        } else if isActive {
            ToolbarItemGroup(placement: .navigation) { historyButtons }
            ToolbarItem(placement: .principal) { searchField }
            ToolbarItemGroup(placement: .primaryAction) { libraryButtons }
        }
    }

    @ViewBuilder private var historyButtons: some View {
        Button { reading.move(-1) } label: { Label("Back", systemImage: "chevron.left") }
            .disabled(!isActive || !reading.history.canGoBack).keyboardShortcut("[").help("Back (⌘[)")
        Button { reading.move(1) } label: { Label("Forward", systemImage: "chevron.right") }
            .disabled(!isActive || !reading.history.canGoForward).keyboardShortcut("]").help("Forward (⌘])")
    }

    @ViewBuilder private var libraryButtons: some View {
        Button { showReferences = true } label: { Label("Saved and Recent", systemImage: "bookmark") }
            .keyboardShortcut("b", modifiers: [.command, .shift]).disabled(!isActive)
            .help("Saved and Recent References (⇧⌘B)")
        Button(action: openLibrary) { Label("Manage Docs", systemImage: "square.and.arrow.down") }
            .help("Install, update, and import documentation")
    }

    @ViewBuilder private var searchField: some View {
        let field = HStack(spacing: 7) {
            Image(systemName: "magnifyingglass").foregroundStyle(.secondary)
            TextField("Search documentation", text: $query)
                .textFieldStyle(.plain).focused($searchFocused)
                .onSubmit(commitSearchSelection)
                .onKeyPress(.downArrow) { moveSelection(1) }
                .onKeyPress(.upArrow) { moveSelection(-1) }
            if !query.isEmpty {
                Button { query = ""; searchFocused = true } label: {
                    Image(systemName: "xmark.circle.fill").foregroundStyle(.secondary)
                }.buttonStyle(.plain).accessibilityLabel("Clear search")
            }
        }
        .frame(minWidth: 240, idealWidth: 420, maxWidth: 520)
        if #available(macOS 26.0, *) {
            // The toolbar gives each item its own glass capsule.
            field.padding(.horizontal, 10).padding(.vertical, 4)
        } else {
            field.padding(.horizontal, 9).padding(.vertical, 5)
                .background(.quaternary, in: RoundedRectangle(cornerRadius: 7, style: .continuous))
        }
    }

    private var scopeBar: some View {
        HStack(spacing: 7) {
            scopeButton(scope == .installed ? stack.map { "Installed · \(displayStack($0))" } ?? "Installed" : "Installed",
                        scope: .installed)
            if projectContext != nil { scopeButton("This Project", scope: .project) }
            scopeButton("Imported", scope: .imported)
            if let currentPack { scopeButton(displayName(currentPack), scope: .docset) }
            Spacer(minLength: 0)
            if scope == .project && projectContext != nil {
                Button("Project Versions") { showProjectReferences = true }.controlSize(.small)
            }
        }.padding(.horizontal, 16).padding(.vertical, 8)
    }

    private func scopeButton(_ title: String, scope item: Scope) -> some View {
        Button { changeScope(item) } label: {
            Text(title).font(.caption.weight(scope == item ? .semibold : .regular)).lineLimit(1)
                .padding(.horizontal, 9).padding(.vertical, 4)
                .background(scope == item ? Color.accentColor.opacity(0.2) : Color.primary.opacity(0.06), in: Capsule())
                .contentShape(Rectangle())
        }.buttonStyle(.plain).accessibilityAddTraits(scope == item ? [.isSelected] : [])
    }

    @ViewBuilder private func sidebar(_ key: SearchKey) -> some View {
        if key.query.isEmpty && key.pack == nil {
            docsets
        } else {
            VStack(alignment: .leading, spacing: 0) {
                if let pack = scopedPack, key.query.isEmpty { browseHeader(pack) }
                if let failure = searchFailure, failure.key == key {
                    sidebarMessage("Couldn't Load Documentation", detail: failure.message)
                        .frame(maxHeight: .infinity)
                } else if committedSearch != key {
                    ProgressView().controlSize(.small).frame(maxWidth: .infinity, maxHeight: .infinity)
                } else if let navigation = cachedNavigation(for: key), !navigation.groups.isEmpty {
                    groupIndex(navigation)
                } else if hits.isEmpty {
                    sidebarMessage(key.query.isEmpty ? "No Articles" : "No Results",
                                   detail: key.query.isEmpty ? nil : "Try a different term or search scope.")
                        .frame(maxHeight: key.query.isEmpty ? nil : .infinity)
                } else {
                    results(browsing: key.query.isEmpty)
                }
            }
        }
    }

    private var docsets: some View {
        let visiblePacks = visiblePacks
        return Group {
            if visiblePacks.isEmpty {
                VStack(spacing: 8) {
                    let bundled = model.catalog?.currentDocs.filter { !$0.installed } ?? []
                    if scope == .installed, !bundled.isEmpty {
                        sidebarMessage("No Docs Installed Yet",
                                       detail: "\(bundled.count) docsets come with Devline. Installing works offline.")
                        Button("Install All") {
                            Task {
                                await model.mutate(bundled.map { CoreRequest("docs.install", id: $0.id, version: $0.version) },
                                                   label: "Installing documentation", completion: "Installed \(bundled.count) docsets.")
                            }
                        }.buttonStyle(.borderedProminent).disabled(model.operation != nil)
                        Button("Choose Docsets…", action: openLibrary).buttonStyle(.link)
                    } else {
                        sidebarMessage(scope == .imported ? "No Imported Docs" : "No Installed Docs")
                        Button(scope == .imported ? "Manage Imports" : "Install Docs", action: openLibrary)
                    }
                }.frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ScrollView {
                    LazyVStack(spacing: 2) {
                        if scope == .project, let projectContext {
                            Text(URL(fileURLWithPath: projectContext.path).lastPathComponent)
                                .font(.callout.weight(.semibold)).foregroundStyle(.secondary)
                                .frame(maxWidth: .infinity, alignment: .leading).padding(.horizontal, 8).padding(.bottom, 4)
                        }
                        ForEach(visiblePacks, id: \.reference) { pack in
                            Button { choosePack(pack, inProject: scope == .project) } label: {
                                HStack(spacing: 8) {
                                    Image(systemName: "book.closed").foregroundStyle(.secondary)
                                    Text(displayName(pack)).lineLimit(1)
                                    Spacer(minLength: 4)
                                    Text(release(pack)).font(.caption).foregroundStyle(.secondary).lineLimit(1)
                                    Image(systemName: "chevron.right").font(.caption2).foregroundStyle(.tertiary)
                                }.padding(8).contentShape(Rectangle())
                            }.buttonStyle(.plain)
                        }
                    }.padding(8)
                }
            }
        }.frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private func browseHeader(_ pack: DocPack) -> some View {
        let currentPackVersions = packs.filter { $0.id == pack.id }
        return HStack {
            if group != nil {
                Button { group = nil; selectedID = nil } label: { Image(systemName: "chevron.left") }
                    .buttonStyle(.borderless).accessibilityLabel("All Sections")
            }
            Text(group.map { $0.isEmpty ? "Other Articles" : $0 } ?? "Contents")
                .font(.callout.weight(.semibold)).lineLimit(1)
                .accessibilityAddTraits(.isHeader)
            Spacer()
            if let entry = pack.entryId {
                Button { open(packEntryID(pack, entry)) } label: { Image(systemName: "list.bullet.rectangle") }
                    .buttonStyle(.borderless).help("Open Contents Page").accessibilityLabel("Open Contents Page")
            }
            if currentPackVersions.count > 1 {
                Menu(release(pack)) {
                    ForEach(currentPackVersions, id: \.reference) { version in
                        Button(release(version)) { choosePack(version, inProject: false) }
                    }
                }.menuStyle(.borderlessButton).fixedSize().accessibilityLabel("Docset Release")
            }
        }.padding(.horizontal, 12).padding(.vertical, 9)
    }

    private func groupIndex(_ navigation: DocsetNavigation) -> some View {
        ScrollView {
            LazyVStack(spacing: 2) {
                ForEach(navigation.groups + (navigation.hasUngrouped ? [""] : []), id: \.self) { name in
                    Button {
                        group = name
                        selectedID = nil
                    } label: {
                        HStack {
                            Text(name.isEmpty ? "Other Articles" : name)
                            Spacer()
                            Image(systemName: "chevron.right").font(.caption2).foregroundStyle(.tertiary)
                        }.padding(8).contentShape(Rectangle())
                    }.buttonStyle(.plain)
                }
            }.padding(8)
        }
    }

    private func results(browsing: Bool) -> some View {
        let packs = packs
        return ScrollViewReader { proxy in
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 2) {
                    ForEach(hits) { hit in
                        let name = packs.first { $0.id == hit.packId && $0.version == hit.version }
                            .map(displayName) ?? displayStack(hit.stack)
                        let title = ReaderMarkup.title(hit.title, fallback: name)
                        let label = browsing ? title : hit.symbol ?? title
                        Button { open(hit) } label: {
                            VStack(alignment: .leading, spacing: 3) {
                                Text(label).font(.callout.weight(.medium)).lineLimit(2)
                                if !browsing {
                                    Text("\(name) · \(displayDocVersion(hit.version))")
                                        .font(.caption).foregroundStyle(.secondary).lineLimit(1)
                                    if let preview = resultPreview(hit, title: title, label: label) {
                                        Text(preview).font(.caption).foregroundStyle(.secondary).lineLimit(2)
                                    }
                                }
                            }
                            .frame(maxWidth: .infinity, alignment: .leading).padding(8)
                            .background(selectedID == hit.id ? Color.accentColor.opacity(0.15) : .clear,
                                        in: RoundedRectangle(cornerRadius: 6))
                            .contentShape(Rectangle())
                        }.buttonStyle(.plain).id(hit.id)
                            .accessibilityAddTraits(selectedID == hit.id ? [.isSelected] : [])
                    }
                    if moreArticles {
                        if pageRequest != nil {
                            ProgressView().controlSize(.small).padding(8)
                        } else {
                            Button(browseError == nil ? "Load More" : "Retry") { Task { await loadMoreArticles() } }
                                .buttonStyle(.borderless).padding(8)
                        }
                    }
                    if let browseError { Text(browseError).font(.caption).foregroundStyle(.secondary).padding(8) }
                }.padding(8)
            }.onChange(of: reveal) { _, _ in if let selectedID { proxy.scrollTo(selectedID) } }
        }
    }

    private func sidebarMessage(_ title: String, detail: String? = nil) -> some View {
        VStack(spacing: 5) {
            Text(title).font(.callout.weight(.medium))
            if let detail { Text(detail).font(.caption).foregroundStyle(.secondary).multilineTextAlignment(.center) }
        }.frame(maxWidth: .infinity).padding(12)
    }

    private func resultPreview(_ hit: SearchHit, title: String, label: String) -> String? {
        let candidates = hit.symbol == nil ? [hit.snippet, hit.matchText] : [hit.matchText, hit.snippet]
        return candidates.compactMap { $0.map(ReaderMarkup.summary) }.first { preview in
            !preview.isEmpty && preview.caseInsensitiveCompare(label) != .orderedSame
                && preview.caseInsensitiveCompare(title) != .orderedSame
        }
    }

    @ViewBuilder private var reader: some View {
        if let document {
            let pack = docset(document.packId, version: document.version)
            let projectPath = projectLinkPath
            let documentFocus = loadedDocument?.requestedFocus == openedLocation?.focusText ? document.focusText : nil
            VStack(spacing: 0) {
                if let linkIssue {
                    HStack(spacing: 10) {
                        Image(systemName: "info.circle").foregroundStyle(.secondary)
                        Text(linkIssue.message).font(.callout)
                        Spacer()
                        if let packID = linkIssue.installPack { Button("Install Docs") { installDocset(packID) } }
                        if let url = linkIssue.source { Link("Open Source", destination: url) }
                        Button { self.linkIssue = nil } label: { Image(systemName: "xmark") }
                            .buttonStyle(.borderless).accessibilityLabel("Dismiss Link Message")
                    }.padding(10).background(.bar)
                    Divider()
                }
                DocReader(document: document, pack: pack, packName: pack.map(displayName) ?? displayStack(document.stack),
                          navigationToken: navigationToken, anchor: openedLocation?.anchor,
                          focus: openedLocation?.focusText ?? documentFocus,
                          restoredPosition: openedLocation?.position, restoredOffset: openedLocation?.anchorOffset,
                          cueSearchMatch: openedLocation?.searchMatch == true,
                          shortcutsActive: isActive,
                          isSaved: reading.isSaved(document.id),
                          save: { anchor, focus in
                              var location = currentLocation(document)
                              location.anchor = anchor
                              location.focusText = focus
                              Task { await reading.toggleSaved(location) }
                          },
                          linkRevision: "\(model.libraryRevision):\(projectPath ?? ""):\(projectPath == nil ? "" : projectPins)",
                          linkStatuses: { urls in
                              try await classifyLinks(urls, in: document, projectPath: projectPath, priority: .utility)
                          },
                          navigate: { url, status in Task { await follow(url, status: status, from: document) } },
                          openPart: { id in open(id, documentContext: true) },
                          reportPosition: { [token = navigationToken] fraction, anchor, offset in
                              guard isActive, token == navigationToken else { return }
                              if let anchor, anchor != currentReaderAnchor { currentReaderAnchor = anchor }
                              reading.updatePosition(id: document.id, fraction: fraction, anchor: anchor, anchorOffset: offset)
                          })
                    .id(document.id)
            }
        } else if let failure = documentFailure, failure.key == readKey {
            ContentUnavailableView("Document Unavailable", systemImage: "doc.text", description: Text(failure.message))
        } else if directDocumentID != nil {
            ProgressView().frame(maxWidth: .infinity, maxHeight: .infinity)
        } else {
            ContentUnavailableView("Documentation", systemImage: "book",
                                   description: Text("Choose a docset or search for a symbol or topic."))
        }
    }

    private func packPrecedes(_ a: DocPack, _ b: DocPack) -> Bool {
        displayName(a).localizedStandardCompare(displayName(b)) == .orderedAscending
    }

    private func docset(_ id: String, version: String) -> DocPack? {
        packs.first { $0.id == id && $0.version == version }
    }

    private func isImported(_ id: String) -> Bool { id.hasPrefix("devdocs-") }

    private func displayName(_ pack: DocPack) -> String {
        let prefix = "DevDocs · "
        return isImported(pack.id) && pack.name.hasPrefix(prefix) ? String(pack.name.dropFirst(prefix.count)) : pack.name
    }

    private func release(_ pack: DocPack) -> String { displayDocVersion(pack.version) }
    private func packEntryID(_ pack: DocPack, _ entry: String) -> String { "\(pack.reference)/\(entry)" }

    private func applyEntryPoint() {
        scope = selectedPack != nil ? .docset : projectScopeRequested && projectContext != nil ? .project : .installed
        group = nil
        browsedPackInProject = projectScopeRequested && scope == .docset
        openedLocation = nil
        currentReaderAnchor = nil
        linkIssue = nil
        pendingBrowseReveal = nil
        restoring = false
        navigationToken = UUID()
        if directDocumentID == nil && trimmedQuery.isEmpty, let pack = currentPack, let entry = pack.entryId {
            open(packEntryID(pack, entry))
        }
    }

    private func changeScope(_ value: Scope) {
        guard scope != value || (value == .installed && stack != nil) else { return }
        if value == .docset, let currentPack {
            packReference = currentPack.reference
            browsedPackInProject = openedLocation?.projectScoped == true
        }
        scope = value
        group = nil
        stack = nil
        selectedID = nil
        pendingBrowseReveal = nil
    }

    private func choosePack(_ pack: DocPack, inProject: Bool) {
        scope = .docset
        stack = pack.stack
        packReference = pack.reference
        query = ""
        group = nil
        browsedPackInProject = inProject
        selectedID = nil
        pendingBrowseReveal = nil
        if let entry = pack.entryId {
            open(packEntryID(pack, entry), projectScoped: inProject)
        } else {
            directDocumentID = nil
            openedLocation = nil
            linkIssue = nil
        }
    }

    private func open(_ hit: SearchHit) {
        selectedID = hit.id
        open(hit.id, anchor: hit.matchAnchor, focus: browsing ? nil : hit.matchText, searchMatch: !browsing)
    }

    private func open(_ id: String, anchor: String? = nil, focus: String? = nil,
                      projectScoped: Bool? = nil, documentContext: Bool = false, searchMatch: Bool = false) {
        let project = projectScoped ?? (scope == .project || (scope == .docset && browsedPackInProject))
        var location = ReferenceLocation(id: id, query: query, stack: stack, pack: packReference,
                                         anchor: anchor, focusText: focus,
                                         browseIndex: browsing ? hits.firstIndex { $0.id == id } : nil)
        location.scope = scope.rawValue
        location.group = group
        location.projectScoped = documentContext ? openedLocation?.projectScoped : project
        location.searchMatch = searchMatch ? true : nil
        reading.open(location)
    }

    private func commitSearchSelection() {
        guard hasCurrentResults else {
            if searching, !trimmedQuery.isEmpty { submittedSearch = searchKey }
            return
        }
        if let hit = hits.first(where: { $0.id == selectedID }) ?? hits.first { open(hit) }
    }

    private func moveSelection(_ offset: Int) -> KeyPress.Result {
        guard hasCurrentResults, !hits.isEmpty else { return .ignored }
        let index = hits.firstIndex { $0.id == selectedID } ?? (offset > 0 ? -1 : hits.count)
        if index + offset >= hits.count, browsing, moreArticles {
            Task { await loadMoreArticles(advanceSelection: true) }
        } else {
            selectedID = hits[min(max(index + offset, 0), hits.count - 1)].id
            reveal = UUID()
        }
        return .handled
    }

    private func cachedNavigation(for key: SearchKey) -> DocsetNavigation? {
        guard let navigation, navigation.key == key.navigationKey else { return nil }
        return navigation.value
    }

    private func search(_ key: SearchKey) async {
        guard key == searchKey, !Task.isCancelled else { return }
        committedSearch = nil
        pageRequest = nil
        browseError = nil
        moreArticles = false
        searchFailure = nil
        if submittedSearch != key { submittedSearch = nil }
        let pack = scopedPack
        guard !key.query.isEmpty || pack != nil else {
            hits = []
            selectedID = nil
            committedSearch = key
            return
        }
        do {
            if !key.query.isEmpty { try await Task.sleep(for: .milliseconds(120)) }
            if let pack, let indexKey = key.navigationKey {
                let index: DocsetNavigation
                if let cached = cachedNavigation(for: key) {
                    index = cached
                } else {
                    index = try await Core.request(CoreRequest("docs.navigation", version: pack.version, packId: pack.id))
                    try Task.checkCancellation()
                    guard key == searchKey else { return }
                    navigation = (indexKey, index)
                }
                if !index.groups.isEmpty {
                    hits = []
                    committedSearch = key
                    return
                }
            }
            let results: [SearchHit] = try await Core.request(
                CoreRequest("docs.search", version: pack?.version, query: key.query, stack: key.stack, packId: pack?.id,
                            limit: key.query.isEmpty ? browseLimit + 1 : searchLimit, category: key.category,
                            group: key.group, projectPath: key.project,
                            kind: key.scope == .docset ? nil : key.scope.rawValue)
            )
            try Task.checkCancellation()
            guard key == searchKey else { return }
            hits = key.query.isEmpty ? Array(results.prefix(browseLimit)) : results
            moreArticles = key.query.isEmpty && results.count > browseLimit
            committedSearch = key
            selectedID = results.contains(where: { $0.id == directDocumentID }) ? directDocumentID : nil
            if selectedID != nil { reveal = UUID() }
            if submittedSearch == key {
                submittedSearch = nil
                commitSearchSelection()
            }
            await revealBrowseHistory()
        } catch is CancellationError {
        } catch {
            guard key == searchKey, !Task.isCancelled else { return }
            searchFailure = (key, error.localizedDescription)
            hits = []
            selectedID = nil
        }
    }

    private func loadMoreArticles(advanceSelection: Bool = false) async {
        guard let pack = scopedPack, browsing, moreArticles, pageRequest == nil, hasCurrentResults else { return }
        let key = searchKey
        let request = UUID()
        let offset = hits.count
        let previousSelection = selectedID
        pageRequest = request
        browseError = nil
        defer { if pageRequest == request { pageRequest = nil } }
        do {
            let next: [SearchHit] = try await Core.request(
                CoreRequest("docs.search", version: pack.version, query: "", packId: pack.id,
                            limit: browseLimit + 1, offset: offset, category: key.category, group: key.group)
            )
            guard pageRequest == request, key == searchKey, hits.count == offset else { return }
            hits.append(contentsOf: next.prefix(browseLimit))
            moreArticles = next.count > browseLimit
            if advanceSelection, selectedID == previousSelection, let first = next.first {
                selectedID = first.id
                reveal = UUID()
            }
        } catch is CancellationError {
        } catch {
            guard pageRequest == request, key == searchKey else { return }
            browseError = error.localizedDescription
        }
    }

    private func revealBrowseHistory() async {
        guard let target = pendingBrowseReveal, browsing, hasCurrentResults, !Task.isCancelled else { return }
        if hits.contains(where: { $0.id == target.id }) {
            selectedID = target.id
            reveal = UUID()
            pendingBrowseReveal = nil
        } else if moreArticles && hits.count <= target.index && browseError == nil {
            await loadMoreArticles()
        } else {
            pendingBrowseReveal = nil
        }
    }

    private func loadDocument(_ key: ReadKey) async {
        guard key == readKey, !Task.isCancelled else { return }
        documentFailure = nil
        guard let id = key.id else {
            loadedDocument = nil
            recordedRead = key
            return
        }
        do {
            let value: Document
            if let document {
                value = document
            } else {
                let requestedFocus = openedLocation?.focusText
                let revision = model.libraryRevision
                value = try await Core.request(CoreRequest("docs.show", id: id, focusText: requestedFocus))
                try Task.checkCancellation()
                guard key == readKey else { return }
                loadedDocument = LoadedDocument(key: key, revision: revision, requestedFocus: requestedFocus, value: value)
            }
            recordedRead = key
            await reading.didOpen(currentLocation(value), restoring: restoring, searchPreview: false)
        } catch is CancellationError {
        } catch {
            guard key == readKey, !Task.isCancelled else { return }
            documentFailure = (key, error.localizedDescription)
        }
    }

    private func currentLocation(_ value: Document) -> ReferenceLocation {
        var location = openedLocation ?? ReferenceLocation(id: value.id)
        if let currentReaderAnchor {
            location.anchor = currentReaderAnchor
            location.focusText = nil
        }
        return location
    }

    private func restore(_ request: ReferenceRequest) {
        let location = request.location
        if let loaded = loadedDocument, loaded.key == readKey, loaded.revision == model.libraryRevision,
           loaded.value.id == location.id,
           location.position != nil || (location.anchor?.isEmpty == false && loaded.value.html?.isEmpty == false) {
            // Keep the reader mounted for history positions and links to this article's own sections.
            loadedDocument?.key = ReadKey(id: location.id, token: request.token)
        }
        query = location.query
        stack = location.stack
        packReference = location.pack
        scope = location.scope.flatMap(Scope.init(rawValue:)) ?? (location.pack == nil ? .installed : .docset)
        if scope == .project && projectContext == nil { scope = .installed }
        group = location.group
        if scope == .docset { browsedPackInProject = location.projectScoped == true }
        selectedID = location.id
        directDocumentID = location.id
        openedLocation = location
        currentReaderAnchor = location.anchor
        navigationToken = request.token
        restoring = request.restoring
        linkIssue = nil
        pendingBrowseReveal = request.restoring && location.pack != nil && trimmedQuery.isEmpty
            ? location.browseIndex.map { (location.id, $0) } : nil
        if reading.request?.token == request.token { reading.request = nil }
    }

    private func openReference(_ reference: ReadingReference) {
        reading.open(ReferenceLocation(id: reference.id, stack: reference.stack,
                                       pack: "\(reference.packId)@\(reference.version)",
                                       anchor: reference.anchor, focusText: reference.focusText))
    }

    private func classifyLinks(_ urls: [String], in document: Document, projectPath: String?,
                               priority: TaskPriority = .userInitiated) async throws -> [DocLinkStatus] {
        try await Core.request(
            CoreRequest("docs.link-status", id: document.id, urls: urls, projectPath: projectPath),
            priority: priority
        )
    }

    private func follow(_ url: String, status: DocLinkStatus?, from document: Document) async {
        let origin = readKey
        do {
            let result: DocLinkStatus?
            if let status { result = status }
            else { result = try await classifyLinks([url], in: document, projectPath: projectLinkPath).first }
            guard origin == readKey, !Task.isCancelled else { return }
            let source = URL(string: url, relativeTo: URL(string: document.sourceUrl))
                .flatMap { safeExternalURL($0.absoluteURL.absoluteString) }
            switch result?.kind {
            case .installed:
                if let id = result?.id { open(id, anchor: result?.anchor, focus: result?.focusText, documentContext: true) }
            case .online:
                if let source { NSWorkspace.shared.open(source) }
            case .missingDocument:
                let packReference = result?.packId.map { id in result?.version.map { "\(id)@\($0)" } ?? id }
                linkIssue = LinkIssue(message: "This page is unavailable in your installed documentation.",
                                      source: source, installPack: result?.canInstall == true ? packReference : nil)
            case .missingSection:
                linkIssue = LinkIssue(message: "This section is unavailable in the installed release.", source: source)
            default:
                linkIssue = LinkIssue(message: "This link is unavailable.")
            }
        } catch {
            guard origin == readKey, !Task.isCancelled else { return }
            linkIssue = LinkIssue(message: "Couldn't check this link. Try again.")
        }
    }

    private enum Scope: String, Equatable { case installed, project, imported, docset }
    private struct SearchKey: Equatable {
        let query: String
        let scope: Scope
        let pack, group, stack, category: String?
        let revision: Int
        let project: String?
        let pins: String

        var navigationKey: NavigationKey? {
            guard query.isEmpty, group == nil, category == nil, let pack else { return nil }
            return NavigationKey(pack: pack, revision: revision)
        }
    }
    private struct NavigationKey: Equatable {
        let pack: String
        let revision: Int
    }
    private struct ReadKey: Equatable {
        let id: String?
        let token: UUID
    }
    private struct LoadedDocument {
        var key: ReadKey
        let revision: Int
        let requestedFocus: String?
        let value: Document
    }
    private struct LinkIssue {
        let message: String
        var source: URL?
        var installPack: String?
    }
}

private struct DocsetNavigation: Decodable, Sendable {
    let groups: [String]
    let hasUngrouped: Bool
}

struct DocLinkStatus: Decodable, Sendable {
    enum Kind: String, Decodable, Sendable {
        case installed, online, missingDocument, missingSection, unavailable

        var label: String {
            switch self {
            case .installed: "Available offline"
            case .online: "Online"
            case .missingDocument: "Page unavailable offline"
            case .missingSection: "Section unavailable offline"
            case .unavailable: "Unavailable"
            }
        }
    }
    let url: String
    let kind: Kind
    let id, anchor, focusText, packId, version: String?
    let canInstall: Bool?
}
