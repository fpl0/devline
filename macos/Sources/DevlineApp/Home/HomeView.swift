import AppKit
import SwiftUI

private func newer(_ a: String, _ b: String) -> Bool {
    a.compare(b, options: .numeric) == .orderedDescending
}

extension Catalog {
    /// The release of each starter a new project should use: its newest verified one.
    var currentStarters: [Starter] {
        var chosen: [String: Starter] = [:]
        for starter in templates where starter.verified {
            if let existing = chosen[starter.id], !newer(starter.version, existing.version) { continue }
            chosen[starter.id] = starter
        }
        return chosen.values.sorted { $0.name.localizedStandardCompare($1.name) == .orderedAscending }
    }

    /// What a starter is, in its author's words. A locally rebuilt release only records that it was
    /// rebuilt, so its description comes from the newest authored release of the same starter.
    func summary(of starter: Starter) -> String {
        let generated = "Locally rebuilt "
        guard starter.description.hasPrefix(generated) else { return starter.description }
        let authored = templates.filter { $0.id == starter.id && !$0.description.hasPrefix(generated) }
            .max { newer($1.version, $0.version) }
        return authored?.description ?? starter.description
    }

    /// One bundled release per docset: the active one, else an installed one, else the newest.
    var currentDocs: [DocPack] {
        var chosen: [String: DocPack] = [:]
        func rank(_ pack: DocPack) -> Int { pack.active == true ? 2 : pack.installed ? 1 : 0 }
        for pack in docs where !pack.localOnly && !pack.id.hasPrefix("devdocs-") {
            if let existing = chosen[pack.id] {
                let better = rank(pack) > rank(existing)
                    || (rank(pack) == rank(existing) && newer(pack.version, existing.version))
                if !better { continue }
            }
            chosen[pack.id] = pack
        }
        return chosen.values.sorted { $0.name.localizedStandardCompare($1.name) == .orderedAscending }
    }
}

/// What is on this Mac right now, and what is bundled but not installed yet.
private struct OfflineKit {
    let docs: [DocPack]
    let starters: [Starter]
    let runtimes: [RuntimeToolchain]?
    let images: Int
    let docker: Bool?

    /// Service images the starters use that are not saved on this Mac yet. Nil until Docker's state is known.
    let unsavedImages: [InfrastructureService]?

    var missingDocs: [DocPack] { docs.filter { !$0.installed } }
    var missingStarters: [Starter] { starters.filter { !$0.isInstalled } }
    var missing: Int { missingDocs.count + missingStarters.count }
    var installed: Int { docs.count + starters.count - missing }
    var requests: [CoreRequest] {
        missingDocs.map { CoreRequest("docs.install", id: $0.id, version: $0.version) }
            + missingStarters.map { CoreRequest("templates.install", id: $0.reference) }
    }
}

struct HomeView: View {
    @ObservedObject var model: WorkspaceModel
    @ObservedObject var inventory: RuntimeInventory
    @ObservedObject var infrastructure: InfrastructureLibrary
    @ObservedObject var shelves: PackageShelves
    let chooseFolder: () -> Void
    let create: (Starter) -> Void
    let showStarter: (String) -> Void
    let showDocs, showServices, showPackages: () -> Void
    /// Opens Toolchains on one runtime, on `docker`, or on whatever was last selected.
    let showToolchain: (String?) -> Void
    let startersRequest: Int
    let online: Bool
    var visible = true

    @Preference("devline.home.stack") private var stackFilter = ""
    @State private var showAllSteps = false
    @State private var commandInstalled = FileManager.default.isExecutableFile(atPath: devlineCommandLocation.path(percentEncoded: false))

    private var kit: OfflineKit {
        let starters = model.catalog?.currentStarters ?? []
        let needed = Set(starters.flatMap { $0.services ?? [] })
        let unsaved = infrastructure.status == nil ? nil : infrastructure.catalog?.services.filter {
            needed.contains($0.reference) && infrastructure.image($0.reference)?.archived != true
        }
        return OfflineKit(docs: model.catalog?.currentDocs ?? [], starters: starters,
                          runtimes: inventory.snapshot?.runtimes, images: infrastructure.archived.count,
                          docker: infrastructure.status?.docker.available, unsavedImages: unsaved)
    }

    var body: some View {
        let kit = kit
        ScrollViewReader { proxy in
            ScrollView {
                VStack(alignment: .leading, spacing: 34) {
                    readiness(kit)
                    gallery(kit).id("starters")
                }
                .padding(.horizontal, 32).padding(.vertical, 28)
                .frame(maxWidth: 1000, alignment: .leading)
                .frame(maxWidth: .infinity)
            }
            .onChange(of: startersRequest) { _, _ in
                withAnimation { proxy.scrollTo("starters", anchor: .top) }
            }
        }
        .task(id: model.completedInfrastructureJobIDs) {
            await inventory.load(force: false)
            await infrastructure.loadIfNeeded(after: model.completedInfrastructureJobIDs)
        }
        .task(id: model.completedPackageJobIDs) {
            await shelves.loadIfNeeded(after: model.completedPackageJobIDs)
        }
        .onPageShown(visible) {
            await inventory.load(force: false)
            await infrastructure.loadIfNeeded(after: model.completedInfrastructureJobIDs)
        }
    }

    // MARK: Readiness

    private enum StepState { case done, todo, optional, checking }

    /// One thing a Mac needs before it loses its connection.
    private struct Step: Identifiable {
        let id: String
        let state: StepState
        let detail: String
        var busy = false
        var action: String?
        var enabled = true
        var perform: () -> Void = {}
    }

    /// Everything this Mac needs before it goes offline. Only what still needs doing is shown
    /// until asked; a Mac that is ready gets one quiet line.
    private func readiness(_ kit: OfflineKit) -> some View {
        let steps = [contentStep(kit), toolchainStep(kit), dockerStep(kit), imageStep(kit), packageStep(kit), commandStep]
        let todo = steps.filter { $0.state == .todo }
        let checking = steps.contains { $0.state == .checking }
        let ready = todo.isEmpty && !checking
        let visible = showAllSteps ? steps : todo
        return VStack(alignment: .leading, spacing: 14) {
            HStack(alignment: .center, spacing: 12) {
                Image(systemName: ready ? "checkmark.seal.fill" : "checklist")
                    .font(.system(size: 24)).foregroundStyle(ready ? Color.green : Color.accentColor)
                    .frame(width: 32).accessibilityHidden(true)
                VStack(alignment: .leading, spacing: 2) {
                    Text(ready ? "Ready to work offline"
                         : todo.isEmpty ? "Checking this Mac…"
                         : "\(todo.count) \(todo.count == 1 ? "step" : "steps") left before you go offline")
                        .font(.title3.weight(.semibold)).accessibilityAddTraits(.isHeader)
                    Text(ready ? "Everything you need is on this Mac."
                         : online ? "Do \(todo.count == 1 ? "this" : "these") once while you’re connected."
                         : "You’re offline. Downloads will wait for a connection.")
                        .font(.callout).foregroundStyle(.secondary)
                }
                Spacer(minLength: 8)
                Button(showAllSteps ? "Show Less" : ready ? "Details" : "Show All") {
                    withAnimation(.snappy(duration: 0.22)) { showAllSteps.toggle() }
                }.buttonStyle(.link).font(.callout)
            }
            if !visible.isEmpty {
                VStack(spacing: 0) {
                    ForEach(visible) { step in
                        if step.id != visible.first?.id { Divider().padding(.leading, 44) }
                        row(step)
                    }
                }
                .insetSurface(radius: 10)
            }
        }
        .padding(18)
        .surface(radius: 14)
    }

    private func row(_ step: Step) -> some View {
        HStack(alignment: .center, spacing: 12) {
            Group {
                switch step.state {
                case .done: Image(systemName: "checkmark.circle.fill").foregroundStyle(.green)
                case .todo: Image(systemName: "circle.dashed").foregroundStyle(.orange)
                case .optional: Image(systemName: "minus.circle").foregroundStyle(.secondary)
                case .checking: ProgressView().controlSize(.small)
                }
            }
            .font(.system(size: 16)).frame(width: 20)
            .accessibilityLabel(step.state == .done ? "Done" : step.state == .todo ? "To do" : step.state == .optional ? "Optional" : "Checking")
            Text(step.id).font(.callout.weight(.medium)).frame(width: 150, alignment: .leading)
            Text(step.detail).font(.callout).foregroundStyle(.secondary).lineLimit(2)
                .fixedSize(horizontal: false, vertical: true)
            Spacer(minLength: 12)
            if step.busy { ProgressView().controlSize(.small) }
            if let action = step.action {
                Button(action, action: step.perform).disabled(!step.enabled)
            }
        }
        .padding(.horizontal, 12).padding(.vertical, 9)
    }

    // Documentation and starters ship inside the app, so this step never needs a connection.
    private func contentStep(_ kit: OfflineKit) -> Step {
        let installing = model.operation != nil
        if kit.missing == 0 {
            return Step(id: "Docs and starters", state: .done,
                        detail: "\(kit.docs.count) docsets, \(kit.starters.count) starters", action: "Open Docs", perform: showDocs)
        }
        return Step(id: "Docs and starters", state: .todo,
                    detail: installing ? "\(model.operation ?? "Installing")…" : "\(summary(kit)) included with Devline. No connection needed.",
                    busy: installing, action: "Install", enabled: !installing) { install(kit) }
    }

    private func toolchainStep(_ kit: OfflineKit) -> Step {
        guard let runtimes = kit.runtimes else {
            return Step(id: "Toolchains", state: .checking, detail: "Looking for compilers and interpreters…")
        }
        let missing = runtimes.filter { !$0.available }
        if missing.isEmpty {
            return Step(id: "Toolchains", state: .done, detail: "All \(runtimes.count) found", action: "View") { showToolchain(nil) }
        }
        return Step(id: "Toolchains", state: missing.count == runtimes.count ? .todo : .optional,
                    detail: "\(runtimes.count - missing.count) of \(runtimes.count) found. Missing \(missing.map { displayStack($0.id) }.joined(separator: ", ")).",
                    action: "How to Install") { showToolchain(missing.first?.id) }
    }

    private func dockerStep(_ kit: OfflineKit) -> Step {
        guard let docker = kit.docker else { return Step(id: "Docker", state: .checking, detail: "Looking for an engine…") }
        if docker { return Step(id: "Docker", state: .done, detail: "Running") }
        return Step(id: "Docker", state: kit.unsavedImages?.isEmpty == false ? .todo : .optional,
                    detail: "Not running. Only projects with a database or other service need it.",
                    action: "Setup Guide") { showToolchain("docker") }
    }

    // Service images come from a registry, so this step needs a connection and Docker.
    private func imageStep(_ kit: OfflineKit) -> Step {
        guard let unsaved = kit.unsavedImages else { return Step(id: "Service images", state: .checking, detail: "Checking saved images…") }
        if unsaved.isEmpty {
            return Step(id: "Service images", state: .done, detail: "\(kit.images) saved", action: "View", perform: showServices)
        }
        if !model.imageQueue.isEmpty {
            let current = model.imageQueue.first.flatMap { reference in unsaved.first { $0.reference == reference } }
            return Step(id: "Service images", state: .todo,
                        detail: "Saving \(current?.name ?? "images") · \(model.imageQueue.count) left",
                        busy: true, action: "Stop", perform: model.stopImageDownloads)
        }
        let bytes = unsaved.reduce(Int64(0)) { $0 + ($1.image.compressedBytes ?? 0) }
        return Step(id: "Service images", state: .todo,
                    detail: "\(unsaved.count) not saved yet\(bytes > 0 ? " · \(storageSize(bytes))" : "")"
                        + (kit.docker == true ? "" : " · start Docker first"),
                    action: "Download All", enabled: kit.docker == true && online) {
            model.downloadImages(unsaved.map(\.reference))
        }
    }

    // Packages come from each runtime's registry, so this step needs a connection.
    private func packageStep(_ kit: OfflineKit) -> Step {
        guard shelves.loaded, let runtimes = kit.runtimes else {
            return Step(id: "Packages", state: .checking, detail: "Checking the package shelves…")
        }
        // Only the runtimes this Mac can actually use.
        let relevant = shelves.stacks.filter { stack in runtimes.contains { $0.id == stack.stack && $0.available } }
        let missing = relevant.filter { !$0.shelf.ready }.map(\.stack)
        if relevant.isEmpty {
            return Step(id: "Packages", state: .optional, detail: "Install a toolchain to keep its packages here.")
        }
        if missing.isEmpty {
            let bytes = relevant.reduce(Int64(0)) { $0 + ($1.shelf.bytes ?? 0) }
            let count = relevant.reduce(0) { $0 + $1.shelf.count }
            return Step(id: "Packages", state: .done,
                        detail: "\(count) packages for \(relevant.count) \(relevant.count == 1 ? "runtime" : "runtimes") · \(storageSize(bytes))",
                        action: "Browse", perform: showPackages)
        }
        if !model.packageQueue.isEmpty {
            return Step(id: "Packages", state: .todo,
                        detail: "Downloading \(model.packageQueue.first.map(displayStack) ?? "") · \(model.packageQueue.count) left",
                        busy: true, action: "Stop") { model.stopDownloads(.packages) }
        }
        return Step(id: "Packages", state: .todo,
                    detail: "The 100 most used per runtime, to add offline. \(relevant.count - missing.count) of \(relevant.count) downloaded.",
                    action: "Download All", enabled: online) { model.download(.packages, missing) }
    }

    // Projects' own `devline.sh` and the terminal use the command. The app itself does not need it.
    private var commandStep: Step {
        if commandInstalled { return Step(id: "Terminal command", state: .done, detail: "Linked at ~/.local/bin/devline") }
        return Step(id: "Terminal command", state: .optional,
                    detail: "Not linked. A project’s ./devline.sh uses it in a terminal.", action: "Install") {
            do {
                try installDevlineCommand()
                commandInstalled = true
                model.notice = "Linked devline into ~/.local/bin. Add that folder to your PATH if it isn’t there."
            } catch { model.error = error.localizedDescription }
        }
    }

    private func summary(_ kit: OfflineKit) -> String {
        let docs = kit.missingDocs.count, starters = kit.missingStarters.count
        let parts = [docs > 0 ? "\(docs) \(docs == 1 ? "docset" : "docsets")" : nil,
                     starters > 0 ? "\(starters) \(starters == 1 ? "starter" : "starters")" : nil].compactMap { $0 }
        return parts.joined(separator: " and ")
    }

    private func install(_ kit: OfflineKit) {
        let requests = kit.requests
        guard !requests.isEmpty else { return }
        Task {
            await model.mutate(requests, label: "Installing documentation and starters",
                               completion: "Installed \(requests.count) \(requests.count == 1 ? "item" : "items"). They are available offline.")
        }
    }

    // MARK: Starters

    /// One runtime's starters at a time, so the page stays short.
    private func gallery(_ kit: OfflineKit) -> some View {
        let stacks = Array(Set(kit.starters.map(\.stack))).sorted {
            displayStack($0).localizedStandardCompare(displayStack($1)) == .orderedAscending
        }
        let selected = stacks.contains(stackFilter) ? stackFilter
            : stacks.first { stack in kit.runtimes?.first { $0.id == stack }?.available != false } ?? stacks.first ?? ""
        let runtime = kit.runtimes?.first { $0.id == selected }
        return VStack(alignment: .leading, spacing: 14) {
            HStack(alignment: .firstTextBaseline) {
                Text("New project").font(.title2.weight(.semibold)).accessibilityAddTraits(.isHeader)
                Spacer()
                Button("Open Folder…", action: chooseFolder)
            }
            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: 6) {
                    ForEach(stacks, id: \.self) { stack in
                        chip(displayStack(stack), selected: selected == stack, tint: stackTint(stack)) { stackFilter = stack }
                    }
                }
            }
            if let runtime, !runtime.available {
                Button { showToolchain(selected) } label: {
                    Label("\(displayStack(selected)) isn’t installed on this Mac. See how to install it.", systemImage: "exclamationmark.triangle.fill")
                        .font(.callout).foregroundStyle(.orange)
                }.buttonStyle(.plain)
            }
            LazyVGrid(columns: [GridItem(.adaptive(minimum: 250, maximum: 420), spacing: 12, alignment: .top)],
                      alignment: .leading, spacing: 12) {
                ForEach(kit.starters.filter { $0.stack == selected }, id: \.reference) { starter in
                    StarterCard(starter: starter, summary: model.catalog?.summary(of: starter) ?? starter.description,
                                note: note(for: starter),
                                create: { create(starter) }, details: { showStarter(starter.reference) })
                }
            }
        }
    }

    private func chip(_ title: String, selected: Bool, tint: Color? = nil, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            HStack(spacing: 6) {
                if let tint { Circle().fill(tint).frame(width: 7, height: 7) }
                Text(title).font(.callout.weight(selected ? .semibold : .regular))
            }
            .padding(.horizontal, 11).padding(.vertical, 5)
            .background(selected ? Color.accentColor.opacity(0.2) : Color.primary.opacity(0.06), in: Capsule())
            .contentShape(Capsule())
        }
        .buttonStyle(.plain)
        .accessibilityAddTraits(selected ? [.isSelected] : [])
    }

    /// The one thing worth knowing before choosing a starter, if anything.
    private func note(for starter: Starter) -> StarterCard.Note? {
        let services = starter.services ?? []
        guard !services.isEmpty else { return nil }
        let names = services.map { reference -> String in
            infrastructure.catalog?.services.first { $0.reference == reference }?.name
                ?? reference.split(separator: "@").first.map { $0.capitalized } ?? reference
        }
        let listed = names.count > 3 ? names.prefix(2).joined(separator: ", ") + " + \(names.count - 2) more" : names.joined(separator: ", ")
        if infrastructure.status != nil {
            let unsaved = services.filter { infrastructure.image($0)?.archived != true }.count
            if unsaved > 0 {
                return .download("\(unsaved) service \(unsaved == 1 ? "image" : "images") to download first")
            }
        }
        return .services(listed)
    }
}

private struct StarterCard: View {
    enum Note {
        case services(String), download(String)
    }

    let starter: Starter
    let summary: String
    let note: Note?
    let create, details: () -> Void
    @State private var hovering = false

    var body: some View {
        Button(action: create) {
            VStack(alignment: .leading, spacing: 9) {
                HStack(alignment: .top, spacing: 10) {
                    Image(systemName: appTypeSymbol(starter.appType))
                        .font(.system(size: 15, weight: .medium)).foregroundStyle(stackTint(starter.stack))
                        .frame(width: 32, height: 32)
                        .background(stackTint(starter.stack).opacity(0.15), in: RoundedRectangle(cornerRadius: 8, style: .continuous))
                        .accessibilityHidden(true)
                    VStack(alignment: .leading, spacing: 2) {
                        Text(starter.name).font(.body.weight(.semibold)).lineLimit(2)
                            .multilineTextAlignment(.leading).fixedSize(horizontal: false, vertical: true)
                        Text(displayAppType(starter.appType)).font(.caption).foregroundStyle(.secondary)
                    }
                    Spacer(minLength: 0)
                    Image(systemName: "plus.circle.fill").font(.system(size: 17))
                        .foregroundStyle(Color.accentColor).opacity(hovering ? 1 : 0).accessibilityHidden(true)
                }
                Text(summary).font(.callout).foregroundStyle(.secondary)
                    .lineLimit(2, reservesSpace: true).multilineTextAlignment(.leading)
                switch note {
                case .services(let text):
                    Label(text, systemImage: "server.rack").font(.caption).foregroundStyle(.secondary).lineLimit(1)
                case .download(let text):
                    Label(text, systemImage: "arrow.down.circle").font(.caption).foregroundStyle(.orange).lineLimit(1)
                case nil:
                    EmptyView()
                }
            }
            .padding(14)
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
            .background(Color.primary.opacity(hovering ? 0.09 : 0.045), in: RoundedRectangle(cornerRadius: 12, style: .continuous))
            .overlay { RoundedRectangle(cornerRadius: 12, style: .continuous).strokeBorder(Color.primary.opacity(hovering ? 0.16 : 0.07)) }
            .contentShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
        }
        .buttonStyle(.plain)
        .onHover { hovering = $0 }
        .animation(.easeOut(duration: 0.12), value: hovering)
        .help("Create a project from \(starter.name)")
        .contextMenu {
            Button("New Project…", action: create)
            Button("Show Starter Details", action: details)
        }
    }
}
