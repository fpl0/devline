import SwiftUI

struct ProjectReferencesView: View {
    @State var project: ProjectContext
    let updated: (ProjectContext) -> Void
    @Environment(\.dismiss) private var dismiss
    @State private var changing: String?
    @State private var error: String?

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Project References").font(.title2.weight(.semibold))
                    Text(URL(fileURLWithPath: project.path).lastPathComponent).foregroundStyle(.secondary)
                }
                Spacer()
                Button("Done") { dismiss() }.keyboardShortcut(.cancelAction)
            }
            Text("Devline matches installed docs to resolved dependency versions. Your chosen versions are remembered for this folder and shared with the CLI.")
                .font(.callout).foregroundStyle(.secondary)
            ScrollView {
                VStack(alignment: .leading, spacing: 18) {
                    ForEach(project.docsets ?? []) { doc in
                        VStack(alignment: .leading, spacing: 8) {
                            HStack {
                                Label(doc.id, systemImage: doc.needsAttention ? "exclamationmark.triangle" : "book.closed")
                                    .font(.headline)
                                Spacer()
                                if changing == doc.id { ProgressView().controlSize(.small) }
                                Picker("Version for \(doc.id)", selection: Binding(
                                    get: { project.pins?.first { $0.id == doc.id }?.version ?? "" },
                                    set: { version in Task { await choose(doc.id, version: version) } }
                                )) {
                                    Text("Match project automatically").tag("")
                                    if let pinned = project.pins?.first(where: { $0.id == doc.id })?.version,
                                       !doc.installedVersions.contains(pinned) {
                                        Text("\(displayDocVersion(pinned)) · Not Installed").tag(pinned)
                                    }
                                    ForEach(doc.installedVersions, id: \.self) { version in Text(displayDocVersion(version)).tag(version) }
                                }.labelsHidden().frame(maxWidth: 290).disabled(changing != nil)
                            }
                            Text(doc.detail).font(.callout).foregroundStyle(doc.needsAttention ? .orange : .secondary)
                            if doc.id == "swiftui", doc.version == nil, doc.availableVersions.isEmpty {
                                Link("Apple’s SwiftUI Reference (Online)", destination: URL(string: "https://developer.apple.com/documentation/swiftui/")!)
                                    .font(.callout)
                            }
                            if let version = doc.version { Text("Using \(displayDocVersion(version))").font(.caption).foregroundStyle(.secondary) }
                        }
                        Divider()
                    }
                    if let dependencies = project.dependencies, !dependencies.isEmpty {
                        Text("Detected Dependencies").font(.headline)
                        ForEach(dependencies) { dependency in
                            HStack(alignment: .top) {
                                Text(dependency.id).font(.callout.monospaced())
                                Spacer()
                                VStack(alignment: .trailing, spacing: 3) {
                                    Text(dependency.version ?? dependency.requirement ?? "Version unspecified")
                                    if let source = dependency.source { Text(source).font(.caption).foregroundStyle(.secondary) }
                                }.textSelection(.enabled)
                            }
                        }
                    } else if (project.docsets ?? []).isEmpty {
                        ContentUnavailableView("No Dependencies Detected", systemImage: "folder", description: Text("Open a folder containing a supported project manifest or lockfile."))
                    }
                }.padding(.vertical, 8)
            }
            if let error { Text(error).font(.callout).foregroundStyle(.red).textSelection(.enabled) }
        }.padding(22).frame(width: 650, height: 540)
    }

    private func choose(_ id: String, version: String) async {
        guard changing == nil else { return }
        changing = id
        error = nil
        defer { changing = nil }
        do {
            let next: ProjectContext = try await Core.request(CoreRequest(version.isEmpty ? "project.unpin" : "project.pin",
                                                                          id: id, version: version.isEmpty ? nil : version, path: project.path))
            project = next
            updated(next)
        } catch { self.error = error.localizedDescription }
    }
}
