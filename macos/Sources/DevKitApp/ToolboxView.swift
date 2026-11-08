import AppKit
import SwiftUI

struct ToolboxView: View {
    @ObservedObject var model: WorkspaceModel
    @ObservedObject var infrastructure: InfrastructureLibrary
    let project: ProjectContext?
    let navigate: (Workspace) -> Void
    let openProject: () -> Void
    let openRecent: (String) -> Void
    @State private var runtimes: [RuntimeToolchain] = []
    @State private var runtimeError: String?

    private var docsCount: Int { Set((model.catalog?.docs ?? []).filter(\.installed).map(\.id)).count }
    private var starterCount: Int { Set((model.catalog?.templates ?? []).filter(\.isInstalled).map(\.id)).count }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 28) {
                VStack(alignment: .leading, spacing: 11) {
                    Label("DEVLINE", systemImage: "shippingbox").font(.caption.weight(.semibold)).foregroundStyle(.secondary)
                    Text("Build wherever you are.").font(.system(size: 30, weight: .semibold))
                    Text("Your code, references, and local services—prepared for the days without a connection.")
                        .font(.title3).foregroundStyle(.secondary).fixedSize(horizontal: false, vertical: true)
                    HStack(spacing: 12) {
                        Button("New Project…") { navigate(.templates) }.buttonStyle(.borderedProminent)
                        Button("Open Project…", action: openProject)
                    }.controlSize(.large).padding(.top, 5)
                }
                if let project {
                    Button { navigate(.project) } label: {
                        HStack(spacing: 14) {
                            Image(systemName: "folder").font(.title2).foregroundStyle(Color.accentColor)
                            VStack(alignment: .leading, spacing: 5) {
                                Text("Continue working").font(.caption).foregroundStyle(.secondary)
                                Text(URL(fileURLWithPath: project.path).lastPathComponent).font(.headline)
                                Text(project.stacks.map(displayStack).joined(separator: " · ")).font(.caption).foregroundStyle(.secondary)
                            }
                            Spacer()
                            Image(systemName: "arrow.right").foregroundStyle(.secondary)
                        }.padding(20).contentShape(Rectangle())
                    }.buttonStyle(.plain).background(.quaternary.opacity(0.35), in: RoundedRectangle(cornerRadius: 14))
                }
                if !model.recentProjects.isEmpty {
                    VStack(alignment: .leading, spacing: 10) {
                        HStack {
                            Text("Recent projects").font(.title3.weight(.semibold))
                            Spacer()
                            Button("Open Another…", action: openProject).controlSize(.small)
                        }
                        ForEach(model.recentProjects.filter { $0.path != project?.path }.prefix(5)) { recent in
                            Button { openRecent(recent.path) } label: {
                                HStack(spacing: 12) {
                                    Image(systemName: "folder").foregroundStyle(.secondary)
                                    VStack(alignment: .leading, spacing: 3) {
                                        Text(recent.name).font(.callout.weight(.medium))
                                        Text((recent.path as NSString).abbreviatingWithTildeInPath).font(.caption).foregroundStyle(.secondary).lineLimit(1).truncationMode(.middle)
                                    }
                                    Spacer()
                                    Image(systemName: "chevron.right").font(.caption).foregroundStyle(.tertiary)
                                }.padding(.vertical, 8).contentShape(Rectangle())
                            }.buttonStyle(.plain).help(recent.path)
                                .contextMenu { Button("Remove from Recent Projects") { model.forgetProject(recent.path) } }
                        }
                        if model.recentProjects.allSatisfy({ $0.path == project?.path }) {
                            Text("Projects you open will appear here, ready to return to.").font(.callout).foregroundStyle(.secondary)
                        }
                    }
                }
                VStack(alignment: .leading, spacing: 16) {
                    HStack {
                        Text("In your toolbox").font(.title3.weight(.semibold))
                        Spacer()
                        Button("Check Offline Readiness") { navigate(.library) }
                    }
                    Grid(horizontalSpacing: 14, verticalSpacing: 14) {
                        GridRow {
                            resource("Documentation", count: "\(docsCount)", detail: "Language and framework docsets", symbol: "book.closed", destination: .docs)
                            resource("App Starters", count: "\(starterCount)", detail: "With packaged dependencies", symbol: "square.stack.3d.up", destination: .templates)
                        }
                        GridRow {
                            resource("Image Archives", count: "\(infrastructure.archived.count)", detail: "\(storageSize(infrastructure.sizeBytes)) · Independent of Docker", symbol: "archivebox", destination: .infrastructure)
                            resource("Toolchains", count: runtimes.isEmpty ? "—" : "\(runtimes.filter(\.available).count)/\(runtimes.count)", detail: runtimeError ?? "Available on this Mac", symbol: "terminal", destination: .runtimes)
                        }
                    }
                }
                VStack(alignment: .leading, spacing: 0) {
                    Text("Prepare once. Keep building.").font(.title3.weight(.semibold)).padding(.bottom, 12)
                    step("1", title: "Choose your stack", detail: "Pick a starter and its references. Verify the toolchain and dependencies on this Mac.", action: "Browse Starters", destination: .templates)
                    Divider().padding(.vertical, 12)
                    step("2", title: "Bring your infrastructure", detail: "Archive databases, queues, search, email, and authentication. Restore the exact images whenever Docker needs them.", action: "Prepare Services", destination: .infrastructure)
                    Divider().padding(.vertical, 12)
                    step("3", title: "Keep a portable copy", detail: "Export prepared releases and image archives for another compatible Mac or a fresh Docker engine.", action: "Open Library", destination: .library)
                }
                if let error = infrastructure.error { Label(error, systemImage: "exclamationmark.circle").font(.caption).foregroundStyle(.orange) }
            }.padding(32).frame(maxWidth: 980, alignment: .leading).frame(maxWidth: .infinity, alignment: .topLeading)
        }
        .task {
            async let refresh: () = infrastructure.refresh()
            do { runtimes = try await Core.request(CoreRequest("runtime.list")); runtimeError = nil }
            catch is CancellationError {}
            catch { runtimeError = error.localizedDescription }
            await refresh
        }
        .onChange(of: model.jobs.filter { !$0.isActive }.map(\.id)) { _, _ in Task { await infrastructure.refresh() } }
    }

    private func resource(_ name: String, count: String, detail: String, symbol: String, destination: Workspace) -> some View {
        Button { navigate(destination) } label: {
            VStack(alignment: .leading, spacing: 12) {
                HStack { Image(systemName: symbol).font(.title3).foregroundStyle(.secondary); Spacer(); Image(systemName: "chevron.right").font(.caption).foregroundStyle(.tertiary) }
                HStack(alignment: .firstTextBaseline, spacing: 10) {
                    Text(count).font(.system(size: 28, weight: .medium, design: .rounded)).monospacedDigit()
                    Text(name).font(.headline)
                }
                Text(detail).font(.caption).foregroundStyle(.secondary).lineLimit(2)
            }.padding(18).frame(maxWidth: .infinity, minHeight: 118, alignment: .topLeading).contentShape(Rectangle())
        }.buttonStyle(.plain).background(.background, in: RoundedRectangle(cornerRadius: 12))
            .overlay { RoundedRectangle(cornerRadius: 12).stroke(.quaternary, lineWidth: 1) }
    }
    private func step(_ number: String, title: String, detail: String, action: String, destination: Workspace) -> some View {
        HStack(alignment: .top, spacing: 14) {
            Text(number).font(.callout.weight(.medium).monospacedDigit()).foregroundStyle(.secondary)
                .frame(width: 28, height: 28).background(.quaternary.opacity(0.5), in: Circle())
            VStack(alignment: .leading, spacing: 5) { Text(title).font(.headline); Text(detail).font(.callout).foregroundStyle(.secondary).fixedSize(horizontal: false, vertical: true) }
            Spacer(minLength: 12)
            Button(action) { navigate(destination) }.controlSize(.small)
        }
    }
}
