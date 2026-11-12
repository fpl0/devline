import SwiftUI

@MainActor
extension WorkspaceModel {
    var completedPackageJobIDs: [String] {
        jobs.filter { !$0.isActive && $0.kind.hasPrefix("packages_") }.map(\.id)
    }

    /// The download in progress for one runtime's shelf, if any.
    func shelfJob(_ stack: String) -> JobRecord? {
        jobs.first { $0.kind == "packages_prepare" && $0.targetReference == stack && $0.isActive }
    }
}

/// The package shelves in the library: what can be added to projects without a connection.
@MainActor
final class PackageShelves: ObservableObject {
    @Published private(set) var stacks: [ShelfStack] = []
    @Published private(set) var loaded = false
    @Published private(set) var error: String?
    private var observedJobs: [String]?
    private var loading: Task<Void, Never>?

    func loadIfNeeded(after completedJobs: [String]) async {
        if loaded, observedJobs == completedJobs { return }
        observedJobs = completedJobs
        await refresh()
    }

    func refresh() async {
        if let loading { return await loading.value }
        let task = Task {
            defer { loading = nil }
            do {
                let catalog: ShelfCatalog = try await Core.request(CoreRequest("packages.catalog"))
                if stacks != catalog.stacks { stacks = catalog.stacks }
                if !loaded { loaded = true }
                if error != nil { error = nil }
            } catch is CancellationError {
            } catch {
                self.error = error.localizedDescription
            }
        }
        loading = task
        await task.value
    }
}

struct PackagesView: View {
    @PageObservedObject var model: WorkspaceModel
    @ObservedObject var shelves: PackageShelves
    @ObservedObject var inventory: RuntimeInventory
    let showToolchain: (String) -> Void
    var visible = true
    @State private var selection: String?
    @State private var search = ""
    @State private var extra = ""
    @State private var removing: ShelfStack?

    private var selected: ShelfStack? { shelves.stacks.first { $0.stack == selection } }
    private func runtime(_ stack: String) -> RuntimeToolchain? { inventory.snapshot?.runtimes.first { $0.id == stack } }
    private var downloadable: [String] {
        shelves.stacks.filter { !$0.shelf.ready && runtime($0.stack)?.available == true }.map(\.stack)
    }

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 10) {
                Text("Popular packages kept on this Mac, so you can add them to a project with no connection.")
                    .font(.callout).foregroundStyle(.secondary)
                Spacer()
                if !model.packageQueue.isEmpty {
                    ProgressView().controlSize(.small)
                    Text("\(model.packageQueue.count) left").font(.callout).foregroundStyle(.secondary)
                    Button("Stop") { model.stopDownloads(.packages) }
                } else if !downloadable.isEmpty {
                    Button("Download All for My Toolchains") { model.download(.packages, downloadable) }
                        .help("Downloads the shelf for every runtime found on this Mac. Needs a connection.")
                }
            }
            .padding(.horizontal, 18).padding(.vertical, 13)
            Divider()
            if let error = shelves.error, shelves.stacks.isEmpty {
                ContentUnavailableView("Couldn’t Read the Package Shelves", systemImage: "shippingbox",
                                       description: Text(error))
            } else if !shelves.loaded {
                ProgressView().frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                HSplitView {
                    List(shelves.stacks, selection: $selection) { stack in
                        HStack(spacing: 9) {
                            Circle().fill(stackTint(stack.stack)).frame(width: 8, height: 8).accessibilityHidden(true)
                            VStack(alignment: .leading, spacing: 3) {
                                Text(displayStack(stack.stack)).font(.body.weight(.medium))
                                Text(status(stack)).font(.caption).foregroundStyle(.secondary)
                            }
                            Spacer(minLength: 0)
                            if model.shelfJob(stack.stack) != nil { ProgressView().controlSize(.small) }
                            else if stack.shelf.ready { Image(systemName: "checkmark.circle").foregroundStyle(.secondary) }
                        }
                        .padding(.vertical, 5).tag(stack.stack)
                    }
                    .listStyle(.inset)
                    .frame(minWidth: 208, idealWidth: 238, maxWidth: 305)
                    Group {
                        if let selected { details(selected).id(selected.stack) }
                        else { ContentUnavailableView("Select a Runtime", systemImage: "shippingbox") }
                    }
                    .frame(minWidth: 380, maxWidth: .infinity, maxHeight: .infinity)
                }
            }
        }
        .task(id: model.completedPackageJobIDs) {
            await inventory.load(force: false)
            await shelves.loadIfNeeded(after: model.completedPackageJobIDs)
            if selected == nil { selection = shelves.stacks.first(where: \.shelf.ready)?.stack ?? shelves.stacks.first?.stack }
        }
        .onPageShown(visible) { await inventory.load(force: false) }
        .confirmationDialog("Remove the \(removing.map { displayStack($0.stack) } ?? "") shelf?",
                            isPresented: Binding(get: { removing != nil }, set: { if !$0 { removing = nil } }),
                            titleVisibility: .visible, presenting: removing) { stack in
            Button("Remove", role: .destructive) {
                Task {
                    await model.mutate([CoreRequest("packages.remove", id: stack.stack)],
                                       label: "Removing packages", completion: "Removed the \(displayStack(stack.stack)) shelf.")
                    await shelves.refresh()
                }
            }
        } message: { _ in
            Text("Projects keep the packages they already use. Adding more will need a new download.")
        }
    }

    private func status(_ stack: ShelfStack) -> String {
        if let job = model.shelfJob(stack.stack) { return job.stage }
        guard stack.shelf.ready else { return "Not downloaded" }
        let size = stack.shelf.bytes.map { " · \(storageSize($0))" } ?? ""
        return "\(stack.shelf.count) of \(stack.shelf.total) packages\(size)"
    }

    private func details(_ stack: ShelfStack) -> some View {
        let job = model.shelfJob(stack.stack)
        let runtime = runtime(stack.stack)
        let term = search.trimmingCharacters(in: .whitespaces).lowercased()
        let visible = stack.packages.filter {
            term.isEmpty || $0.name.lowercased().contains(term) || ($0.summary ?? "").lowercased().contains(term)
                || ($0.category ?? "").lowercased().contains(term)
        }
        let categories = Dictionary(grouping: visible) { $0.category ?? "Other" }
        return VStack(alignment: .leading, spacing: 0) {
            VStack(alignment: .leading, spacing: 10) {
                HStack(alignment: .firstTextBaseline) {
                    VStack(alignment: .leading, spacing: 4) {
                        Text("\(displayStack(stack.stack)) Packages").font(.title2.weight(.semibold))
                        Text(summary(stack)).font(.callout).foregroundStyle(.secondary)
                    }
                    Spacer()
                    if let job {
                        Button("Details") { model.inspectedJob = job }
                    } else if runtime?.available == false {
                        Button("Install \(displayStack(stack.stack)) First") { showToolchain(stack.stack) }
                    } else {
                        Button(stack.shelf.ready ? "Check for Newer Versions" : "Download") { download(stack.stack) }
                            .buttonStyle(.borderedProminent).disabled(!model.packageQueue.isEmpty)
                        if stack.shelf.ready {
                            Menu {
                                Button("Remove from This Mac…") { removing = stack }
                            } label: { Image(systemName: "ellipsis.circle") }
                                .menuIndicator(.hidden).fixedSize().accessibilityLabel("More shelf actions")
                        }
                    }
                }
                if let job {
                    HStack(spacing: 8) {
                        ProgressView().controlSize(.small)
                        Text("\(job.stage). This can take several minutes; you can keep working.")
                            .font(.callout).foregroundStyle(.secondary)
                    }
                }
                HStack(spacing: 8) {
                    Image(systemName: "magnifyingglass").foregroundStyle(.secondary)
                    TextField("Filter \(stack.packages.count) packages", text: $search).textFieldStyle(.plain)
                }
                .padding(.horizontal, 10).padding(.vertical, 6)
                .background(.quaternary, in: RoundedRectangle(cornerRadius: 7))
            }
            .padding(.horizontal, 22).padding(.top, 20).padding(.bottom, 12)
            Divider()
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 0, pinnedViews: [.sectionHeaders]) {
                    ForEach(categories.keys.sorted(), id: \.self) { category in
                        Section {
                            ForEach(categories[category] ?? []) { package in
                                row(package, ready: stack.shelf.ready)
                                Divider().padding(.leading, 22)
                            }
                        } header: {
                            Text(category).font(.caption.weight(.semibold)).foregroundStyle(.secondary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .padding(.horizontal, 22).padding(.vertical, 5).background(.bar)
                        }
                    }
                    if visible.isEmpty {
                        Text("No packages match “\(search)”.").font(.callout).foregroundStyle(.secondary).padding(22)
                    }
                }
            }
            if stack.stack != "swift", runtime?.available != false {
                Divider()
                HStack(spacing: 8) {
                    TextField("Keep another \(stack.registry) package on the shelf", text: $extra)
                        .textFieldStyle(.roundedBorder).onSubmit { addExtra(stack.stack) }
                    Button("Add to Shelf") { addExtra(stack.stack) }
                        .disabled(extra.trimmingCharacters(in: .whitespaces).isEmpty || job != nil)
                        .help("Downloads this package and its dependencies now. Needs a connection.")
                }
                .padding(.horizontal, 22).padding(.vertical, 10)
            }
        }
    }

    private func summary(_ stack: ShelfStack) -> String {
        guard stack.shelf.ready else {
            return "\(stack.shelf.total) packages from \(stack.registry). Download them while you’re connected."
        }
        var text = "\(stack.shelf.count) of \(stack.shelf.total) packages from \(stack.registry) on this Mac"
        if let bytes = stack.shelf.bytes { text += " · \(storageSize(bytes))" }
        if let prepared = stack.shelf.preparedAt {
            text += " · downloaded \(Date(timeIntervalSince1970: prepared).formatted(.relative(presentation: .named)))"
        }
        return text
    }

    private func row(_ package: ShelfStack.Package, ready: Bool) -> some View {
        HStack(alignment: .firstTextBaseline, spacing: 12) {
            VStack(alignment: .leading, spacing: 2) {
                Text(package.name).font(.system(.callout, design: .monospaced).weight(.medium)).textSelection(.enabled)
                if let summary = package.summary { Text(summary).font(.caption).foregroundStyle(.secondary) }
            }
            Spacer(minLength: 8)
            if let version = package.version {
                Text(version).font(.caption.monospacedDigit()).foregroundStyle(.secondary)
            } else if ready {
                Label("Unavailable", systemImage: "exclamationmark.circle").font(.caption).foregroundStyle(.orange)
                    .help(package.error ?? "This package could not be downloaded for the local toolchain.")
            }
        }
        .padding(.horizontal, 22).padding(.vertical, 7)
    }

    private func download(_ stack: String, extra: [String]? = nil) {
        Task {
            await model.startJob(CoreRequest("jobs.start", id: stack, kind: "packages_prepare",
                                             packages: extra, timeoutSeconds: 7200), inspect: false)
        }
    }

    private func addExtra(_ stack: String) {
        let name = extra.trimmingCharacters(in: .whitespaces)
        guard !name.isEmpty else { return }
        extra = ""
        download(stack, extra: [name])
    }
}
