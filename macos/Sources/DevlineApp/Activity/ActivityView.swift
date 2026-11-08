import AppKit
import SwiftUI

struct ActivitySheet: View {
    @Environment(\.dismiss) private var dismiss
    @ObservedObject var model: WorkspaceModel
    @State private var selectedJob: JobRecord?

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Text("Activity").font(.title2.weight(.semibold)).accessibilityAddTraits(.isHeader)
                Spacer()
                Button("Done") { dismiss() }.keyboardShortcut(.cancelAction)
            }.padding(18)
            Divider()
            history
        }
        .frame(width: 640, height: 475)
        .task { await model.refreshJobs() }
        .sheet(item: $selectedJob) { JobDetailsView(job: $0, model: model) }
    }

    private var history: some View {
        Group {
            if model.jobs.isEmpty {
                ContentUnavailableView("No Activity Yet", systemImage: "clock", description: Text("Downloads, starter builds, verification and bundle transfers appear here with their full logs."))
            } else {
                List(model.jobs) { job in
                    Button { selectedJob = job } label: {
                        HStack(spacing: 12) {
                            Image(systemName: job.icon).foregroundStyle(job.needsAttention ? Color.orange : .secondary)
                            VStack(alignment: .leading, spacing: 5) {
                                Text(job.title).font(.headline)
                                if let path = job.targetPath { Text(URL(fileURLWithPath: path).lastPathComponent).font(.callout.weight(.medium)) }
                                else if let reference = job.targetReference { Text(reference).font(.callout.weight(.medium)) }
                                Text(job.stage).font(.callout).foregroundStyle(.secondary)
                                if let error = job.error { Text(error).font(.caption).foregroundStyle(.orange).lineLimit(2) }
                            }
                            Spacer()
                            VStack(alignment: .trailing, spacing: 5) {
                                Text(job.stateLabel).font(.caption)
                                Text(Date(timeIntervalSince1970: job.createdAt), style: .relative).font(.caption).foregroundStyle(.secondary)
                            }
                        }.padding(.vertical, 7).contentShape(Rectangle())
                    }.buttonStyle(.plain)
                }.listStyle(.inset)
            }
        }
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
            if let path = job.targetPath { Text((path as NSString).abbreviatingWithTildeInPath).font(.callout).foregroundStyle(.secondary).textSelection(.enabled) }
            else if let reference = job.targetReference { Text(reference).font(.callout).foregroundStyle(.secondary) }
            HStack {
                Text(job.stage).font(.headline)
                Spacer()
                Text(cancelling && job.isActive ? "Cancelling…" : job.stateLabel).foregroundStyle(.secondary)
            }
            if job.isActive {
                if let completed = job.completedBytes, let total = job.totalBytes, total > 0 {
                    ProgressView(value: Double(min(completed, total)), total: Double(total))
                    Text("\(storageSize(Int64(completed))) of \(storageSize(Int64(total)))")
                        .font(.caption).foregroundStyle(.secondary)
                } else { ProgressView().progressViewStyle(.linear) }
            }
            if let failure = error ?? job.error { Text(failure).foregroundStyle(.orange).textSelection(.enabled) }
            if job.result?.needsAttention == true {
                Text("Some items still need attention. Completed work has been kept.").font(.callout).foregroundStyle(.orange)
                ScrollView {
                    VStack(alignment: .leading, spacing: 6) {
                        ForEach(Array((job.result?.issues ?? []).enumerated()), id: \.offset) { _, step in
                            Text("\(step.name ?? step.id ?? step.kind ?? "Item"): \(step.detail ?? "Needs attention")")
                                .font(.caption).foregroundStyle(.secondary).textSelection(.enabled)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }
                    }
                }.frame(maxHeight: 100)
            }
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
