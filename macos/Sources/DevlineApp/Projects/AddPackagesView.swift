import SwiftUI

/// Adds packages from the offline shelf to a project, with no connection needed.
struct AddPackagesView: View {
    @ObservedObject var model: WorkspaceModel
    let project: ProjectContext
    let done: () -> Void
    @Environment(\.dismiss) private var dismiss
    @State private var stack: ShelfStack?
    @State private var loadError: String?
    @State private var chosen: Set<String> = []
    @State private var search = ""
    @State private var jobID: String?
    @State private var failure: String?

    private var job: JobRecord? { jobID.flatMap { id in model.jobs.first { $0.id == id } } }
    private var adding: Bool { job?.isActive == true }
    private var download: JobRecord? { stack.flatMap { model.shelfJob($0.stack) } }
    private var name: String { URL(fileURLWithPath: project.path).lastPathComponent }

    var body: some View {
        VStack(spacing: 0) {
            HStack(alignment: .top, spacing: 12) {
                Image(systemName: "shippingbox").font(.system(size: 24)).foregroundStyle(Color.accentColor)
                    .frame(width: 34).accessibilityHidden(true)
                VStack(alignment: .leading, spacing: 3) {
                    Text("Add Packages to \(name)").font(.title2.weight(.semibold))
                    Text("From the shelf on this Mac. Devline adds them, then tests the project with the network blocked.")
                        .font(.callout).foregroundStyle(.secondary)
                }
                Spacer()
            }.padding(20)
            Divider()
            content.frame(maxWidth: .infinity, maxHeight: .infinity)
            Divider()
            footer
        }
        .frame(width: 620, height: 560)
        .interactiveDismissDisabled(adding)
        .task(id: model.completedPackageJobIDs) { await load() }
        .onChange(of: job?.state) { _, state in finish(state) }
    }

    @ViewBuilder private var content: some View {
        if let stack {
            if !stack.shelf.ready {
                VStack(spacing: 12) {
                    if let download {
                        ProgressView()
                        Text(download.stage).font(.headline)
                        Text("This can take several minutes. You can close this and keep working.")
                            .font(.callout).foregroundStyle(.secondary)
                    } else {
                        Image(systemName: "arrow.down.circle").font(.system(size: 34)).foregroundStyle(.secondary)
                        Text("The \(displayStack(stack.stack)) shelf isn’t on this Mac yet").font(.headline)
                        Text("Download \(stack.shelf.total) popular \(stack.registry) packages once, while you’re connected. After that, adding them works anywhere.")
                            .font(.callout).foregroundStyle(.secondary).multilineTextAlignment(.center).frame(maxWidth: 400)
                        Button("Download Now") {
                            Task {
                                await model.startJob(CoreRequest("jobs.start", id: stack.stack, kind: "packages_prepare",
                                                                 timeoutSeconds: 7200), inspect: false)
                            }
                        }.buttonStyle(.borderedProminent)
                    }
                }.padding(30)
            } else {
                list(stack)
            }
        } else if let loadError {
            ContentUnavailableView("Packages Unavailable", systemImage: "shippingbox", description: Text(loadError))
        } else {
            ProgressView()
        }
    }

    private func list(_ stack: ShelfStack) -> some View {
        let term = search.trimmingCharacters(in: .whitespaces).lowercased()
        let visible = stack.packages.filter {
            term.isEmpty || $0.name.lowercased().contains(term) || ($0.summary ?? "").lowercased().contains(term)
                || ($0.category ?? "").lowercased().contains(term)
        }
        let categories = Dictionary(grouping: visible) { $0.category ?? "Other" }
        return VStack(spacing: 0) {
            HStack(spacing: 8) {
                Image(systemName: "magnifyingglass").foregroundStyle(.secondary)
                TextField("Search \(stack.shelf.count) \(stack.registry) packages", text: $search).textFieldStyle(.plain)
            }
            .padding(.horizontal, 10).padding(.vertical, 6)
            .background(.quaternary, in: RoundedRectangle(cornerRadius: 7))
            .padding(.horizontal, 20).padding(.vertical, 10)
            Divider()
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 0, pinnedViews: [.sectionHeaders]) {
                    ForEach(categories.keys.sorted(), id: \.self) { category in
                        Section {
                            ForEach(categories[category] ?? []) { package in
                                row(package)
                                Divider().padding(.leading, 46)
                            }
                        } header: {
                            Text(category).font(.caption.weight(.semibold)).foregroundStyle(.secondary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .padding(.horizontal, 20).padding(.vertical, 5).background(.bar)
                        }
                    }
                    if visible.isEmpty {
                        Text("Nothing on the shelf matches “\(search)”. You can keep more packages on the shelf in Library → Packages.")
                            .font(.callout).foregroundStyle(.secondary).padding(20)
                    }
                }
            }
            .disabled(adding)
        }
    }

    private func row(_ package: ShelfStack.Package) -> some View {
        let used = package.installed == true
        return Toggle(isOn: Binding(
            get: { used || chosen.contains(package.name) },
            set: { if $0 { chosen.insert(package.name) } else { chosen.remove(package.name) } }
        )) {
            HStack(alignment: .firstTextBaseline, spacing: 10) {
                VStack(alignment: .leading, spacing: 2) {
                    Text(package.name).font(.system(.callout, design: .monospaced).weight(.medium))
                    if let summary = package.summary { Text(summary).font(.caption).foregroundStyle(.secondary) }
                }
                Spacer(minLength: 8)
                Text(used ? "In project" : package.version ?? "Not on shelf")
                    .font(.caption.monospacedDigit()).foregroundStyle(.secondary)
            }
        }
        .toggleStyle(.checkbox)
        .disabled(used || !package.available)
        .padding(.horizontal, 20).padding(.vertical, 7)
    }

    private var footer: some View {
        HStack(spacing: 10) {
            if adding {
                ProgressView().controlSize(.small)
                Text(job?.stage ?? "Starting…").font(.callout).foregroundStyle(.secondary).lineLimit(2)
            } else if let failure {
                Image(systemName: "exclamationmark.circle").foregroundStyle(.orange)
                Text(failure).font(.caption).foregroundStyle(.secondary).lineLimit(3).textSelection(.enabled)
            }
            Spacer(minLength: 8)
            if failure != nil, !adding {
                Button("Try With the Network") { add(offline: false) }
                    .help("Lets the package manager fetch anything the shelf is missing")
            }
            Button(adding ? "Stop" : "Close") {
                if adding, let jobID {
                    Task { let _: Acknowledgment? = try? await Core.request(CoreRequest("jobs.cancel", jobId: jobID)) }
                } else { dismiss() }
            }.keyboardShortcut(.cancelAction)
            Button(chosen.count > 1 ? "Add \(chosen.count) Packages" : "Add Package") { add(offline: true) }
                .buttonStyle(.borderedProminent).keyboardShortcut(.defaultAction)
                .disabled(chosen.isEmpty || adding)
        }
        .padding(16)
    }

    private func load() async {
        do {
            let catalog: ShelfCatalog = try await Core.request(CoreRequest("packages.catalog", path: project.path))
            guard !Task.isCancelled else { return }
            stack = catalog.stacks.first
            loadError = stack == nil ? "This project’s runtime has no package shelf." : nil
        } catch is CancellationError {
        } catch { loadError = error.localizedDescription }
    }

    private func add(offline: Bool) {
        guard !chosen.isEmpty, !adding else { return }
        failure = nil
        Task {
            do {
                let started: JobRecord = try await Core.request(CoreRequest(
                    "jobs.start", path: project.path, kind: "packages_add", offline: offline, packages: chosen.sorted()))
                jobID = started.id
                await model.refreshJobs(startedJobID: started.id)
            } catch { failure = error.localizedDescription }
        }
    }

    private func finish(_ state: String?) {
        guard let job, !job.isActive else { return }
        if state == "succeeded" {
            model.notice = "Added \(chosen.sorted().joined(separator: ", ")) to \(name)."
            done()
            dismiss()
        } else if state == "cancelled" {
            failure = "Stopped. The project was not changed."
            jobID = nil
        } else {
            failure = job.error ?? "The packages could not be added. The project was not changed."
            jobID = nil
        }
    }
}
