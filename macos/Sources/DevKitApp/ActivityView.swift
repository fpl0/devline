import AppKit
import SwiftUI

struct JobRecord: Decodable, Identifiable, Sendable {
    let id, kind, state, stage: String
    let createdAt, updatedAt: Double
    let error, logPath, log: String?
    let offset, nextOffset, completedBytes, totalBytes: Int?
    var isActive: Bool { state == "queued" || state == "running" }
    var title: String {
        switch kind {
        case "build": "Refresh Starter Dependencies"
        case "verify": "Verify Offline Readiness"
        case "prepare": "Prepare Starter"
        case "project_prepare": "Prepare Project for Offline Work"
        case "infra_prepare": "Prepare Image Archive"
        case "infra_restore": "Restore Image to Docker"
        case "infra_verify": "Verify Image Archive"
        case "infra_update": "Update Image Archive"
        case "infra_stack_up": "Start Project Services"
        case "infra_stack_down": "Stop Project Services"
        case "infra_export": "Export Image Archives"
        case "infra_import": "Import Image Archives"
        case "docs_prepare": "Prepare Local Documentation"
        case "docs_install": "Download Documentation"
        case "bundle_export": "Export Offline Bundle"
        case "bundle_import": "Import Offline Bundle"
        default: kind.capitalized
        }
    }
    var icon: String { isActive ? "arrow.triangle.2.circlepath" : state == "succeeded" ? "checkmark.circle" : state == "cancelled" ? "stop.circle" : "exclamationmark.circle" }
}

struct ActivityView: View {
    @ObservedObject var model: WorkspaceModel
    var body: some View {
        Group {
            if model.jobs.isEmpty {
                ContentUnavailableView("No Activity Yet", systemImage: "clock", description: Text("Downloads, starter builds, verification and bundle transfers appear here with their full logs."))
            } else {
                List(model.jobs) { job in
                    Button { model.inspectedJob = job } label: {
                        HStack(spacing: 12) {
                            Image(systemName: job.icon).foregroundStyle(job.state == "failed" ? Color.orange : .secondary)
                            VStack(alignment: .leading, spacing: 5) {
                                Text(job.title).font(.headline)
                                Text(job.stage).font(.callout).foregroundStyle(.secondary)
                                if let error = job.error { Text(error).font(.caption).foregroundStyle(.orange).lineLimit(2) }
                            }
                            Spacer()
                            VStack(alignment: .trailing, spacing: 5) {
                                Text(job.state.capitalized).font(.caption)
                                Text(Date(timeIntervalSince1970: job.createdAt), style: .relative).font(.caption).foregroundStyle(.secondary)
                            }
                        }.padding(.vertical, 7).contentShape(Rectangle())
                    }.buttonStyle(.plain)
                }.listStyle(.inset)
            }
        }.task { await model.refreshJobs() }
    }
}

struct JobDetailsView: View {
    @State var job: JobRecord
    @ObservedObject var model: WorkspaceModel
    @Environment(\.dismiss) private var dismiss
    @State private var log = ""
    @State private var offset = 0
    @State private var error: String?
    @State private var cancelling = false
    @State private var truncated = false

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            HStack {
                Label(job.title, systemImage: job.icon).font(.title2.weight(.semibold))
                Spacer()
                Button(job.isActive ? "Run in Background" : "Done") { dismiss() }.keyboardShortcut(.cancelAction)
            }
            HStack {
                Text(job.stage).font(.headline)
                Spacer()
                Text(cancelling && job.isActive ? "Cancelling…" : job.state.capitalized).foregroundStyle(.secondary)
            }
            if job.isActive {
                if let completed = job.completedBytes, let total = job.totalBytes, total > 0 {
                    ProgressView(value: Double(min(completed, total)), total: Double(total))
                    Text("\(ByteCountFormatter.string(fromByteCount: Int64(completed), countStyle: .file)) of \(ByteCountFormatter.string(fromByteCount: Int64(total), countStyle: .file))")
                        .font(.caption).foregroundStyle(.secondary)
                } else { ProgressView().progressViewStyle(.linear) }
            }
            if let failure = error ?? job.error { Text(failure).foregroundStyle(.orange).textSelection(.enabled) }
            ScrollView([.vertical, .horizontal]) {
                Text(log.isEmpty ? "Waiting for output…" : log)
                    .font(.system(.caption, design: .monospaced)).textSelection(.enabled)
                    .frame(maxWidth: .infinity, alignment: .topLeading).padding(12)
            }.background(.background, in: RoundedRectangle(cornerRadius: 8))
                .overlay(RoundedRectangle(cornerRadius: 8).stroke(.quaternary))
                .accessibilityLabel("Job Output")
            HStack {
                if let path = job.logPath {
                    Button("Open Full Log") { NSWorkspace.shared.open(URL(fileURLWithPath: path)) }
                    Button("Show Log in Finder") { NSWorkspace.shared.activateFileViewerSelecting([URL(fileURLWithPath: path)]) }
                }
                if truncated { Text("Showing recent output").font(.caption).foregroundStyle(.secondary) }
                Spacer()
                if job.isActive { Button("Cancel Job", role: .destructive) { Task { await cancel() } }.disabled(cancelling) }
            }
        }.padding(22).frame(width: 760, height: 570)
            .task(id: job.id) { await follow() }
    }

    private func follow() async {
        while !Task.isCancelled {
            do {
                let next: JobRecord = try await Core.request(CoreRequest("jobs.status", offset: offset, jobId: job.id))
                let previous = offset
                job = next
                log += next.log ?? ""
                offset = next.nextOffset ?? offset
                if log.utf8.count > 250_000 { log = String(log.suffix(180_000)); truncated = true }
                // Drain all pending chunks, including the tail written just before exit.
                if !next.isActive && offset == previous { await model.refresh(); return }
                if offset - previous < 60_000 { try await Task.sleep(for: .milliseconds(500)) }
            } catch is CancellationError { return }
            catch { self.error = error.localizedDescription; return }
        }
    }

    private func cancel() async {
        cancelling = true
        do { let _: Acknowledgment = try await Core.request(CoreRequest("jobs.cancel", jobId: job.id)) }
        catch { self.error = error.localizedDescription; cancelling = false }
    }
}
