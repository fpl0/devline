import AppKit
import SwiftUI

struct TemplatesView: View {
    @ObservedObject var model: WorkspaceModel
    @Binding var selection: String?
    @Binding var newStarter: Starter?
    let openDocs: (DocPack) -> Void
    @State private var stack: String?
    @State private var appType: String?
    @State private var buildCandidate: Starter?

    private var starters: [Starter] { model.catalog?.templates ?? [] }
    private var stacks: [String] { Array(Set(starters.map(\.stack))).sorted() }
    private var types: [String] { Array(Set(starters.map(\.appType))).sorted() }
    private var filtered: [Starter] {
        starters.filter { (stack == nil || $0.stack == stack) && (appType == nil || $0.appType == appType) }
    }
    private var families: [StarterFamily] {
        Dictionary(grouping: filtered, by: \.id).values
            .compactMap(StarterFamily.init)
            .sorted { $0.latest.name.localizedStandardCompare($1.latest.name) == .orderedAscending }
    }
    private var selectedStarter: Starter? { filtered.first { $0.reference == selection } }
    private var familySelection: Binding<String?> {
        Binding(
            get: { selectedStarter?.id },
            set: { id in
                if id != selectedStarter?.id {
                    selection = families.first { $0.id == id }?.preferred.reference
                }
            }
        )
    }

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 10) {
                Picker("App Type", selection: $appType) {
                    Text("All Types").tag(nil as String?)
                    ForEach(types, id: \.self) { Text(displayAppType($0)).tag(Optional($0)) }
                }
                .labelsHidden()
                .fixedSize()
                .accessibilityLabel("Filter templates by app type")
                Picker("Stack", selection: $stack) {
                    Text("All Stacks").tag(nil as String?)
                    ForEach(stacks, id: \.self) { Text(displayStack($0)).tag(Optional($0)) }
                }
                .labelsHidden()
                .fixedSize()
                .accessibilityLabel("Filter templates by stack")
                Spacer()
            }
            .padding(.horizontal, 18)
            .padding(.vertical, 13)
            Divider()
            if families.isEmpty {
                ContentUnavailableView(
                    "No Starters",
                    systemImage: "square.on.square",
                    description: Text(starters.isEmpty ? "The catalog has no starters yet." : "Choose another app type or stack.")
                )
            } else {
                HSplitView {
                    List(families, selection: familySelection) { family in
                        familyRow(family).tag(family.id)
                    }
                    .listStyle(.inset)
                    .frame(minWidth: 208, idealWidth: 238, maxWidth: 305)
                    Group {
                        if let starter = selectedStarter, let family = families.first(where: { $0.id == starter.id }) {
                            templateDetails(starter, in: family)
                        } else {
                            ContentUnavailableView("Select a Starter", systemImage: "square.on.square")
                        }
                    }
                    .frame(minWidth: 350, maxWidth: .infinity, maxHeight: .infinity)
                }
            }
        }
        .onAppear(perform: selectVisibleStarter)
        .onChange(of: filtered.map(\.reference)) { _, _ in selectVisibleStarter() }
        .confirmationDialog("Refresh template dependencies?", isPresented: Binding(
            get: { buildCandidate != nil }, set: { if !$0 { buildCandidate = nil } }
        ), titleVisibility: .visible, presenting: buildCandidate) { starter in
            Button("Build New Release") {
                Task {
                    await model.startJob(CoreRequest("jobs.start", id: starter.reference, kind: "build"))
                }
            }
        } message: { starter in
            Text("Resolve current dependencies for \(starter.name) online, then verify the new release offline before installing it.")
        }
    }

    private func familyRow(_ family: StarterFamily) -> some View {
        HStack(spacing: 10) {
            Image(systemName: appTypeSymbol(family.latest.appType))
                .foregroundStyle(.secondary)
                .frame(width: 20)
                .accessibilityHidden(true)
            VStack(alignment: .leading, spacing: 3) {
                Text(family.latest.name).font(.body.weight(.medium))
                Text("\(displayStack(family.latest.stack)) · \(displayAppType(family.latest.appType))")
                    .font(.caption).foregroundStyle(.secondary)
            }
            Spacer(minLength: 0)
            if family.releases.contains(where: \.isInstalled) {
                Image(systemName: "checkmark.circle").foregroundStyle(.secondary)
                    .help("At least one release is installed")
                    .accessibilityLabel("Installed releases")
            }
        }
        .padding(.vertical, 5)
    }

    private func templateDetails(_ starter: Starter, in family: StarterFamily) -> some View {
        let docs = relatedDocs(for: starter)
        return ScrollView {
            VStack(alignment: .leading, spacing: 19) {
                VStack(alignment: .leading, spacing: 7) {
                    Text("\(displayStack(starter.stack)) · \(displayAppType(starter.appType))")
                        .font(.caption.weight(.semibold)).foregroundStyle(.secondary)
                    Text(starter.name).font(.title2.weight(.semibold))
                    Text(starter.description).foregroundStyle(.secondary)
                        .fixedSize(horizontal: false, vertical: true)
                    if let minimumMacos = starter.minimumMacos {
                        Label("macOS \(minimumMacos) or later", systemImage: "desktopcomputer")
                            .font(.caption).foregroundStyle(.secondary)
                    }
                }
                actions(for: starter, docs: docs)
                Divider()
                releaseRow(starter, in: family)
                StarterInventoryView(starter: starter, model: model)
                if starter.verified || !docs.isEmpty {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Included").font(.headline)
                        if starter.verified {
                            Label("Packaged dependencies and an offline test", systemImage: "shippingbox")
                        }
                        if !docs.isEmpty {
                            ForEach(docs, id: \.reference) { pack in
                                HStack {
                                    Label(pack.name, systemImage: "book.closed")
                                    Text(displayDocVersion(pack.version)).font(.caption).foregroundStyle(.secondary)
                                    Spacer()
                                    if pack.installed {
                                        Button("Read Docs") { openDocs(pack) }.controlSize(.small)
                                            .accessibilityLabel("Read \(pack.name) \(displayDocVersion(pack.version))")
                                    } else {
                                        Button("Prepare Docs") { Task { await model.startJob(CoreRequest("jobs.start", id: pack.id, version: pack.version, kind: "docs_prepare")) } }.controlSize(.small)
                                    }
                                }
                            }
                        }
                    }
                    .font(.callout)
                    .foregroundStyle(.secondary)
                }
                DisclosureGroup("Technical Details") {
                    VStack(alignment: .leading, spacing: 6) {
                        Text("Toolchain: \(starter.toolchain) \(starter.minimumVersion)")
                        Text("Target: \(starter.target)")
                        if let minimumMacos = starter.minimumMacos { Text("macOS: \(minimumMacos) or later") }
                    }
                    .font(.callout).foregroundStyle(.secondary)
                    .padding(.top, 6)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .textSelection(.enabled)
                }
                .font(.callout)
            }
            .frame(maxWidth: 580, alignment: .leading)
            .padding(26)
            .frame(maxWidth: .infinity, alignment: .topLeading)
        }
        .id(starter.id)
    }

    private func releaseRow(_ starter: Starter, in family: StarterFamily) -> some View {
        HStack(alignment: .center, spacing: 13) {
            VStack(alignment: .leading, spacing: 4) {
                Text("Release").font(.headline)
                Text(releaseStatus(starter)).font(.callout).foregroundStyle(.secondary)
            }
            Spacer(minLength: 0)
            if family.releases.count > 1 {
                Picker("Template release", selection: $selection) {
                    ForEach(family.releases, id: \.reference) { release in
                        Text(release.version + (release.isInstalled ? " · Installed" : "") + (release.verified ? "" : " · Preview"))
                            .tag(Optional(release.reference))
                    }
                }
                .labelsHidden()
                .fixedSize()
                .accessibilityLabel("Template release")
            } else {
                Text(starter.version).font(.callout.monospacedDigit())
            }
        }
    }

    private func actions(for starter: Starter, docs: [DocPack]) -> some View {
        HStack(spacing: 9) {
            if starter.verified {
                Button("New Project…") { newStarter = starter }
                    .buttonStyle(.borderedProminent)
                if !starter.isInstalled {
                    Button("Install") { install(starter, withDocs: !starter.docs.isEmpty) }
                        .help("Install the starter and its documentation now. Creating a project also does this.")
                }
            } else {
                Text("Project creation requires an offline-verified release.")
                    .font(.callout).foregroundStyle(.secondary)
            }
            if starter.isInstalled && docs.contains(where: { !$0.installed }) {
                Button("Install Related Docs") { install(starter, withDocs: true) }
            }
            Menu {
                if !starter.isInstalled && !starter.docs.isEmpty {
                    Button("Install Template Only") { install(starter, withDocs: false) }
                }
                Button("Refresh Dependencies…") { buildCandidate = starter }
                if starter.isInstalled {
                    Button("Verify Offline Now") { Task { await model.startJob(CoreRequest("jobs.start", id: starter.reference, kind: "verify")) } }
                }
            } label: {
                Label("More Actions", systemImage: "ellipsis.circle")
                    .labelStyle(.iconOnly)
            }
            .menuStyle(.borderlessButton)
            .fixedSize()
            .accessibilityLabel("More Template Actions")
        }
        .disabled(model.operation != nil)
    }

    private func relatedDocs(for starter: Starter) -> [DocPack] {
        let packs = model.catalog?.docs ?? []
        return starter.docs.compactMap { reference in
            return packs.first { $0.reference == reference }
                ?? packs.filter { $0.id == reference }.max {
                    $0.version.compare($1.version, options: .numeric) == .orderedAscending
                }
        }
    }

    private func releaseStatus(_ starter: Starter) -> String {
        if !starter.verified { return starter.isInstalled ? "Installed preview" : "Preview" }
        return starter.isInstalled ? "Installed · Packaged dependencies" : "Available · Packaged dependencies"
    }

    private func selectVisibleStarter() {
        if selectedStarter == nil {
            selection = families.first?.preferred.reference
        }
    }

    private func install(_ starter: Starter, withDocs: Bool) {
        Task {
            if withDocs {
                await model.startJob(CoreRequest("jobs.start", id: starter.reference, kind: "prepare"))
            } else {
                await model.mutate([CoreRequest("templates.install", id: starter.reference)],
                                   label: "Installing \(starter.name)", completion: "Installed \(starter.name) \(starter.version).")
            }
        }
    }

    private struct StarterFamily: Identifiable {
        let id: String
        let latest: Starter
        let releases: [Starter]

        var preferred: Starter { releases.first(where: \.verified) ?? latest }

        init?(_ starters: [Starter]) {
            let ordered = starters.sorted { $0.version.compare($1.version, options: .numeric) == .orderedDescending }
            guard let latest = ordered.first else { return nil }
            self.id = latest.id
            self.latest = latest
            self.releases = ordered
        }
    }
}
