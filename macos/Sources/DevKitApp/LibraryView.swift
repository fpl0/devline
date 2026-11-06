import SwiftUI

struct LibraryView: View {
    @ObservedObject var model: WorkspaceModel
    @State private var selections: Set<Choice> = []
    @State private var removePack: DocPack?

    var body: some View {
        VStack(spacing: 0) {
            if let catalog = model.catalog {
                header
                Divider()
                List {
                    Section("Documentation") {
                        ForEach(catalog.docs, id: \.reference) { pack in
                            choiceRow(.doc(pack), name: pack.name, version: pack.version,
                                      detail: pack.description, installed: pack.installed)
                        }
                    }
                    Section("Starter Templates") {
                        ForEach(catalog.templates, id: \.reference) { starter in
                            choiceRow(.starter(starter), name: starter.name, version: starter.version,
                                      detail: "\(displayStack(starter.stack)) · \(starter.target)", installed: starter.isInstalled)
                        }
                    }
                }
                .listStyle(.inset)
                .onChange(of: availableChoices(in: catalog)) { _, choices in selections.formIntersection(choices) }
            } else {
                ContentUnavailableView {
                    Label("Library Unavailable", systemImage: "tray")
                } description: {
                    Text("Load the catalog to see available doc packs and templates.")
                } actions: {
                    Button("Reload Catalog") { Task { await model.refresh() } }
                }
            }
        }
        .confirmationDialog("Remove this doc pack?", isPresented: Binding(
            get: { removePack != nil }, set: { if !$0 { removePack = nil } }
        ), titleVisibility: .visible, presenting: removePack) { pack in
            Button("Remove \(pack.name)", role: .destructive) {
                Task {
                    await model.mutate([CoreRequest("docs.remove", id: pack.id, version: pack.version)],
                                       label: "Removing \(pack.name)", completion: "Removed \(pack.name) \(pack.version).")
                }
            }
        } message: { pack in
            Text("\(pack.name) \(pack.version) will no longer appear in offline search.")
        }
    }

    private var header: some View {
        HStack(alignment: .center, spacing: 12) {
            VStack(alignment: .leading, spacing: 4) {
                Text("Offline Library").font(.title2.weight(.semibold))
                Text("Select doc packs and starters from any stack to install together.")
                    .font(.callout).foregroundStyle(.secondary)
            }
            Spacer()
            Button {
                Task { await model.refresh() }
            } label: {
                Image(systemName: "arrow.clockwise")
            }
            .help("Refresh library")
            .accessibilityLabel("Refresh library")
            .disabled(model.operation != nil)
            Button(selections.isEmpty ? "Install Selected" : "Install Selected (\(selections.count))") {
                let count = selections.count
                let requests = selections.sorted { $0.sortKey < $1.sortKey }.map(\.request)
                Task {
                    await model.mutate(requests, label: "Installing selected packs",
                                       completion: "Installed \(count) \(count == 1 ? "pack" : "packs").")
                }
            }
            .buttonStyle(.borderedProminent)
            .disabled(selections.isEmpty || model.operation != nil)
        }
        .padding(18)
    }

    private func choiceRow(_ choice: Choice, name: String, version: String, detail: String, installed: Bool) -> some View {
        HStack(spacing: 10) {
            if installed {
                Image(systemName: "checkmark.circle.fill").foregroundStyle(.green)
                    .frame(width: 16)
                    .accessibilityHidden(true)
                title(name, detail: detail)
            } else {
                Toggle(isOn: Binding(
                    get: { selections.contains(choice) },
                    set: { if $0 { selections.insert(choice) } else { selections.remove(choice) } }
                )) {
                    title(name, detail: detail)
                }
                .toggleStyle(.checkbox)
                .disabled(model.operation != nil)
            }
            Spacer(minLength: 12)
            if installed { Text("Installed").font(.caption).foregroundStyle(.secondary) }
            Text(version).font(.callout.monospacedDigit()).foregroundStyle(.secondary)
                .frame(minWidth: 54, alignment: .trailing)
            if installed, case .doc(let pack) = choice {
                Button {
                    removePack = pack
                } label: {
                    Image(systemName: "ellipsis.circle")
                }
                .buttonStyle(.borderless)
                .help("Remove \(name) \(version)")
                .accessibilityLabel("Remove \(name) \(version)")
                .disabled(model.operation != nil)
            }
        }
        .padding(.vertical, 5)
    }

    private func title(_ name: String, detail: String) -> some View {
        VStack(alignment: .leading, spacing: 3) {
            Text(name).font(.body.weight(.medium))
            Text(detail).font(.caption).foregroundStyle(.secondary).lineLimit(2)
        }
    }

    private func availableChoices(in catalog: Catalog) -> Set<Choice> {
        Set(catalog.docs.filter { !$0.installed }.map(Choice.doc) +
            catalog.templates.filter { !$0.isInstalled }.map(Choice.starter))
    }

    private enum Choice: Hashable {
        case doc(DocPack)
        case starter(Starter)

        var request: CoreRequest {
            switch self {
            case .doc(let pack): CoreRequest("docs.install", id: pack.id, version: pack.version)
            case .starter(let starter): CoreRequest("templates.install", id: starter.reference)
            }
        }

        var sortKey: String {
            switch self {
            case .doc(let pack): "0:\(pack.reference)"
            case .starter(let starter): "1:\(starter.reference)"
            }
        }
    }
}
