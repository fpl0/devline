import SwiftUI

struct ImageUpdateSnapshot: Decodable, Sendable {
    struct Item: Decodable, Identifiable, Sendable {
        let id, name, state, detail: String
        let currentVersion, availableVersion, reference: String?
        var canUpdate: Bool { state == "available" && reference != nil }
    }
    let checkedAt: Double?
    let items: [Item]
}

struct ImageUpdatesView: View {
    @ObservedObject var model: WorkspaceModel
    @State private var snapshot: ImageUpdateSnapshot?
    @State private var checking = false
    @State private var error: String?
    @State private var submitting = false
    private var updating: Bool { submitting || model.jobs.contains { $0.kind == "infra_update" && $0.isActive } }
    private var available: [ImageUpdateSnapshot.Item] { snapshot?.items.filter(\.canUpdate) ?? [] }
    var body: some View {
        VStack(spacing: 0) {
            HStack(alignment: .top, spacing: 16) {
                VStack(alignment: .leading, spacing: 5) {
                    Text("Infrastructure Updates").font(.title2.weight(.semibold))
                    Text("Download newer publisher images into your offline library. Existing projects and their data stay on their pinned releases.")
                        .font(.callout).foregroundStyle(.secondary)
                }
                Spacer()
                if checking { ProgressView().controlSize(.small) }
                Button("Check Publishers") { Task { await load(online: true) } }.disabled(checking)
                Button("Update All (\(available.count))") {
                    Task {
                        submitting = true
                        defer { submitting = false }
                        for item in available { await update(item) }
                    }
                }.buttonStyle(.borderedProminent).disabled(checking || updating || available.isEmpty)
            }.padding(20)
            Divider()
            List(snapshot?.items ?? []) { item in
                HStack(spacing: 14) {
                    Image(systemName: item.canUpdate ? "arrow.down.circle.fill" : item.state == "up_to_date" ? "checkmark.circle" : "clock")
                        .foregroundStyle(item.canUpdate ? Color.accentColor : .secondary).font(.title3)
                    VStack(alignment: .leading, spacing: 5) {
                        Text(item.name).font(.headline)
                        if let current = item.currentVersion {
                            Text(item.availableVersion.map { $0 == current ? current : "\(current) → \($0)" } ?? current)
                                .font(.callout.monospacedDigit()).foregroundStyle(.secondary)
                        }
                        Text(item.detail).font(.caption).foregroundStyle(.secondary).fixedSize(horizontal: false, vertical: true)
                    }
                    Spacer()
                    if item.canUpdate { Button("Update Image") { Task { submitting = true; defer { submitting = false }; await update(item) } }.buttonStyle(.borderedProminent).disabled(updating) }
                    else { Text(item.state.replacingOccurrences(of: "_", with: " ").capitalized).font(.caption).foregroundStyle(.secondary) }
                }.padding(.vertical, 8)
            }.listStyle(.inset)
            Divider()
            HStack {
                if let error { Label(error, systemImage: "exclamationmark.circle").foregroundStyle(.orange).textSelection(.enabled) }
                else if let checked = snapshot?.checkedAt { Text("Last checked \(Date(timeIntervalSince1970: checked).formatted(date: .abbreviated, time: .shortened))") }
                else { Text("Publisher checks use the internet. Saved update results and prepared images stay available offline.") }
                Spacer()
            }.font(.caption).foregroundStyle(.secondary).padding(16)
        }.task { await load(online: false) }
            .onChange(of: model.jobs.filter { !$0.isActive }.map(\.id)) { _, _ in Task { await load(online: false) } }
            .onChange(of: model.libraryRevision) { _, _ in Task { await load(online: false) } }
    }
    private func load(online: Bool) async {
        guard !checking else { return }
        checking = true; error = nil
        defer { checking = false }
        do { snapshot = try await Core.request(CoreRequest(online ? "infra.updates.check" : "infra.updates")) }
        catch is CancellationError {}
        catch { self.error = error.localizedDescription }
    }
    private func update(_ item: ImageUpdateSnapshot.Item) async {
        guard let reference = item.reference else { return }
        await model.startJob(CoreRequest("jobs.start", id: reference, kind: "infra_update"))
    }
}
