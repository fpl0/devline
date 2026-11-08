import SwiftUI

private struct BundleInspection: Decodable, Sendable {
    struct Item: Decodable, Sendable {
        let id, name, version: String
        var reference: String { "\(id)@\(version)" }
    }
    struct Problem: Decodable, Sendable { let id, reason: String }
    let compatible, requiresTrust: Bool
    let problems: [Problem]
    let warnings: [Problem]?
    let docs, starters: [Item]
    let sizeBytes: Int64
}

struct BundleImportView: View {
    let path: String
    @ObservedObject var model: WorkspaceModel
    @Environment(\.dismiss) private var dismiss
    @State private var inspection: BundleInspection?
    @State private var error: String?
    @State private var trust = false

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text("Import Offline Bundle").font(.title2.weight(.semibold))
            Text(URL(fileURLWithPath: path).lastPathComponent).foregroundStyle(.secondary)
            if let inspection {
                Text("\(inspection.docs.count) docsets · \(inspection.starters.count) starters · \(storageSize(inspection.sizeBytes))")
                ScrollView {
                    VStack(alignment: .leading, spacing: 10) {
                        ForEach(inspection.docs, id: \.reference) { Text("\($0.name) · \($0.version)") }
                        ForEach(inspection.starters, id: \.reference) { Text("\($0.name) · \($0.version)") }
                        ForEach(Array(inspection.problems.enumerated()), id: \.offset) { _, problem in
                            Label("\(problem.id): \(problem.reason)", systemImage: "exclamationmark.circle").foregroundStyle(.orange)
                        }
                        ForEach(Array((inspection.warnings ?? []).enumerated()), id: \.offset) { _, warning in
                            Text(warning.reason).foregroundStyle(.orange)
                        }
                    }.frame(maxWidth: .infinity, alignment: .leading)
                }
                if inspection.requiresTrust {
                    Toggle("I trust the source of these starter scripts", isOn: $trust)
                    Text("This bundle includes starters outside Devline’s bundled catalog. Import copies their files; verification and project commands can execute their scripts.")
                        .font(.caption).foregroundStyle(.secondary)
                }
                Text("Toolchains are not transferred. Verify starters on this Mac after importing.").font(.caption).foregroundStyle(.secondary)
            } else if error == nil { ProgressView("Checking every file and compatibility…") }
            if let error { Text(error).foregroundStyle(.orange).textSelection(.enabled) }
            HStack {
                Spacer()
                Button("Cancel") { dismiss() }.keyboardShortcut(.cancelAction)
                Button("Import") {
                    dismiss()
                    Task { await model.startJob(CoreRequest("jobs.start", path: path, kind: "bundle_import", trustStarters: trust)) }
                }.buttonStyle(.borderedProminent).disabled(inspection?.compatible != true || (inspection?.requiresTrust == true && !trust))
            }
        }.padding(22).frame(width: 620, height: 430)
            .task {
                do { inspection = try await Core.request(CoreRequest("bundle.inspect", path: path)) }
                catch { self.error = error.localizedDescription }
            }
    }
}
