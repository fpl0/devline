import SwiftUI

struct ImageImportView: View {
    struct Inspection: Decodable, Sendable {
        struct Image: Decodable, Sendable {
            let id, version, platform, source, imageId: String
            let sizeBytes: Int64
            let known: Bool
            let detail: String?
            var reference: String { "\(id)@\(version)" }
        }
        let images: [Image]
        let requiresTrust: Bool
        let manifestSha256: String
    }
    @Environment(\.dismiss) private var dismiss
    let path: String
    @ObservedObject var model: WorkspaceModel
    @State private var inspection: Inspection?
    @State private var trust = false
    @State private var error: String?
    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text("Import Image Archives").font(.title2.weight(.semibold))
            Text(URL(fileURLWithPath: path).lastPathComponent).font(.callout).foregroundStyle(.secondary)
            if let inspection {
                List(inspection.images, id: \.reference) { image in
                    VStack(alignment: .leading, spacing: 5) {
                        HStack { Text(image.reference).font(.headline); Spacer(); Text(storageSize(image.sizeBytes)).foregroundStyle(.secondary) }
                        Text(image.source).font(.caption.monospaced()).textSelection(.enabled)
                        Text("\(image.platform) · \(image.known ? "Known publisher pin" : "New publisher pin")").font(.caption).foregroundStyle(.secondary)
                    }.padding(.vertical, 5)
                }.listStyle(.inset)
                if inspection.requiresTrust {
                    Toggle("I trust the source of this image bundle", isOn: $trust)
                    Text("This bundle includes publisher versions that this Mac has not checked. Devline verifies the image contents and checksums before saving them. Importing does not start containers.")
                        .font(.caption).foregroundStyle(.secondary)
                }
            } else if error == nil { ProgressView("Inspecting image archives…").frame(maxWidth: .infinity, maxHeight: .infinity) }
            if let error { Text(error).foregroundStyle(.orange).textSelection(.enabled) }
            HStack {
                Spacer()
                Button("Cancel") { dismiss() }.keyboardShortcut(.cancelAction)
                Button("Import") {
                    dismiss()
                    Task { await model.startJob(CoreRequest("jobs.start", path: path, kind: "infra_import", trustImages: trust, manifestSha256: inspection?.manifestSha256)) }
                }.buttonStyle(.borderedProminent).disabled(inspection == nil || (inspection?.requiresTrust == true && !trust))
            }
        }.padding(22).frame(width: 640, height: 460)
            .task {
                do { inspection = try await Core.request(CoreRequest("infra.import.inspect", path: path)) }
                catch { self.error = error.localizedDescription }
            }
    }
}
