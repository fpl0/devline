import SwiftUI

private struct StarterInventory: Decodable, Sendable {
    struct Entry: Decodable, Identifiable, Sendable { let name, kind: String; var id: String { name } }
    struct Dependency: Decodable, Identifiable, Sendable { let name, version: String; var id: String { name + "@" + version } }
    struct Verification: Decodable, Sendable { let at: Double; let toolchain, target: String }
    let sizeBytes, fileCount: Int
    let contents: [Entry]
    let dependencies: [Dependency]
    let verifiedAt: Double?
    let verification: Verification?
    let readyOffline: Bool
}

struct StarterInventoryView: View {
    let starter: Starter
    @ObservedObject var model: WorkspaceModel
    @State private var inventory: StarterInventory?
    @State private var error: String?
    @State private var remove = false
    @State private var dependencyQuery = ""
    private var key: String { starter.reference + ":" + String(starter.isInstalled) + ":" + model.jobs.filter { $0.state == "succeeded" }.map(\.id).joined() }

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            if let inventory {
                HStack {
                    Text(storageSize(Int64(inventory.sizeBytes))).font(.headline)
                    Text("\(inventory.fileCount) files").foregroundStyle(.secondary)
                    Spacer()
                    if starter.isInstalled { Button("Remove This Version…", role: .destructive) { remove = true }.controlSize(.small) }
                }
                if let verification = inventory.verification {
                    Label("Verified on this Mac \(Date(timeIntervalSince1970: verification.at).formatted(date: .abbreviated, time: .shortened))", systemImage: "checkmark.shield")
                        .font(.callout).foregroundStyle(.green)
                } else {
                    Label("Not yet verified with this Mac’s current toolchain", systemImage: "clock")
                        .font(.callout).foregroundStyle(.secondary)
                }
                if let date = inventory.verifiedAt {
                    Text("Release passed offline tests \(Date(timeIntervalSince1970: date).formatted(date: .abbreviated, time: .shortened))")
                        .font(.caption).foregroundStyle(.secondary)
                }
                DisclosureGroup("Project Contents (\(inventory.contents.count))") {
                    VStack(alignment: .leading, spacing: 6) {
                        ForEach(inventory.contents) { entry in
                            Label(entry.name, systemImage: entry.kind == "directory" ? "folder" : "doc")
                        }
                    }.font(.callout.monospaced()).padding(.top, 7).frame(maxWidth: .infinity, alignment: .leading)
                }
                DisclosureGroup("Packaged Dependencies (\(inventory.dependencies.count))") {
                    LazyVStack(alignment: .leading, spacing: 6) {
                        if !inventory.dependencies.isEmpty {
                            TextField("Find a packaged library", text: $dependencyQuery).textFieldStyle(.roundedBorder)
                                .accessibilityLabel("Find a packaged library").padding(.bottom, 6)
                        }
                        if inventory.dependencies.isEmpty { Text("This starter uses the runtime’s standard library.").foregroundStyle(.secondary) }
                        ForEach(inventory.dependencies.filter { dependencyQuery.isEmpty || $0.name.localizedCaseInsensitiveContains(dependencyQuery) }) { dependency in
                            HStack { Text(dependency.name); Spacer(); Text(dependency.version).foregroundStyle(.secondary) }
                        }
                    }.font(.callout.monospaced()).textSelection(.enabled).padding(.top, 7)
                }
            } else if let error { Text(error).font(.caption).foregroundStyle(.orange) }
            else { ProgressView("Inspecting starter…").controlSize(.small) }
        }
        .task(id: key) {
            inventory = nil; error = nil
            do { inventory = try await Core.request(CoreRequest("templates.inspect", id: starter.reference)) }
            catch is CancellationError {}
            catch { self.error = error.localizedDescription }
        }
        .confirmationDialog("Remove \(starter.name) \(starter.version)?", isPresented: $remove, titleVisibility: .visible) {
            Button("Remove Version", role: .destructive) {
                Task { await model.mutate([CoreRequest("templates.remove", id: starter.reference)], label: "Removing starter", completion: "Removed \(starter.name) \(starter.version).") }
            }
        } message: { Text("This removes the installed starter files. Projects already created from this version are kept.") }
    }
}
