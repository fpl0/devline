import SwiftUI

struct ProjectReferencesView: View {
    @State var project: ProjectContext
    @ObservedObject var model: WorkspaceModel
    let updated: (ProjectContext) -> Void
    @Environment(\.dismiss) private var dismiss
    @State private var changing: String?
    @State private var error: String?
    @State private var contextRequest = UUID()

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
                            if let job = referenceJob(doc), job.isActive {
                                HStack { ProgressView().controlSize(.small); Text(job.stage).font(.caption).foregroundStyle(.secondary) }
                            } else if canPrepare(doc) {
                                Button(doc.id == "swiftui" ? "Prepare from Xcode" : "Prepare \(doc.id) Reference") {
                                    Task { await prepare(doc) }
                                }.controlSize(.small).disabled(changing != nil)
                            }
                            if let job = referenceJob(doc), job.needsAttention, let failure = job.error {
                                Text(failure).font(.caption).foregroundStyle(.orange).textSelection(.enabled)
                            }
                            if doc.id == "swiftui", doc.version == nil, doc.availableVersions.isEmpty {
                                Text("Read Apple’s API declarations and comments from the Xcode SDK installed on this Mac.").font(.caption).foregroundStyle(.secondary)
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
            .onChange(of: model.libraryRevision) { _, _ in Task { await refresh() } }
            .onChange(of: model.jobs.filter { !$0.isActive }.map(\.id)) { _, _ in Task { await refresh() } }
    }

    private func referenceJob(_ doc: ProjectDocset) -> JobRecord? {
        model.jobs.first { ["docs_prepare", "docs_install"].contains($0.kind) && ($0.targetReference == doc.id || $0.targetReference?.hasPrefix(doc.id + "@") == true) }
    }
    private func canPrepare(_ doc: ProjectDocset) -> Bool {
        if let version = doc.prepareVersion { return !doc.installedVersions.contains(version) }
        return doc.id == "swiftui" && doc.version == nil && doc.installedVersions.isEmpty
    }
    private func prepare(_ doc: ProjectDocset) async {
        guard changing == nil else { return }
        changing = doc.id
        defer { changing = nil }
        let version = doc.prepareVersion
        let local = doc.id == "swiftui" || model.catalog?.docs.contains { $0.id == doc.id && $0.version == version } == true
        await model.startJob(CoreRequest("jobs.start", id: doc.id, version: version, kind: local ? "docs_prepare" : "docs_install"), inspect: false)
    }
    private func refresh() async {
        guard changing == nil else { return }
        let token = UUID()
        contextRequest = token
        do {
            let next: ProjectContext = try await Core.request(CoreRequest("project.context", path: project.path))
            guard !Task.isCancelled, contextRequest == token, changing == nil else { return }
            project = next
            updated(next)
        } catch is CancellationError {} catch { self.error = error.localizedDescription }
    }

    private func choose(_ id: String, version: String) async {
        guard changing == nil else { return }
        changing = id
        contextRequest = UUID()
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
