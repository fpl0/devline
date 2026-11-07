import AppKit
import SwiftUI
import UniformTypeIdentifiers

struct InfrastructureView: View {
    @ObservedObject var model: WorkspaceModel
    @ObservedObject var library: InfrastructureLibrary
    let openDocs: (String) -> Void
    @State private var selection: String?
    @State private var query = ""
    @State private var archivedOnly = false
    @State private var removing: InfrastructureService?
    @State private var importPath: String?
    @State private var helpRequest: HelpRequest?

    private struct HelpRequest: Identifiable {
        let id: String
    }

    private var services: [InfrastructureService] {
        (library.catalog?.services ?? []).filter {
            (!archivedOnly || library.image($0.reference)?.archived == true) &&
            (query.isEmpty || "\($0.name) \($0.category) \($0.description)".localizedCaseInsensitiveContains(query))
        }
    }

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 12) {
                TextField("Find a database, queue, or service", text: $query).textFieldStyle(.roundedBorder)
                    .frame(maxWidth: 340).accessibilityLabel("Find infrastructure")
                Picker("Show services", selection: $archivedOnly) {
                    Text("All Services").tag(false)
                    Text("Archived").tag(true)
                }.pickerStyle(.segmented).fixedSize()
                Spacer()
                Menu {
                    Button("Import Image Bundle…", action: importImages)
                    Button("Export All Archived Images…") { export(library.archived.map(\.reference)) }
                        .disabled(library.archived.isEmpty)
                } label: { Label("Transfer", systemImage: "square.and.arrow.up") }
                Button { Task { await library.refresh() } } label: { Image(systemName: "arrow.clockwise") }
                    .help("Refresh image and Docker status").accessibilityLabel("Refresh infrastructure").disabled(library.loading)
            }.padding(16)
            Divider()
            if let error = library.error {
                Label(error, systemImage: "exclamationmark.circle").font(.callout).foregroundStyle(.orange)
                    .padding(16).frame(maxWidth: .infinity, alignment: .leading)
            }
            if library.loading && library.catalog == nil {
                ProgressView("Reading your image library…").frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if services.isEmpty {
                ContentUnavailableView(archivedOnly ? "No Archived Images" : "No Matching Services", systemImage: "server.rack",
                    description: Text(archivedOnly ? "Choose All Services to prepare the infrastructure you want available offline." : "Try another service name or category."))
            } else {
                HSplitView {
                    List(services, id: \.reference, selection: $selection) { service in
                        HStack(spacing: 12) {
                            Image(systemName: service.symbol).font(.title3).foregroundStyle(.secondary).frame(width: 24)
                            VStack(alignment: .leading, spacing: 4) {
                                Text(service.name).fontWeight(.medium)
                                Text("\(service.version) · \(service.category)").font(.caption).foregroundStyle(.secondary)
                            }
                            Spacer()
                            if library.image(service.reference)?.archived == true {
                                Image(systemName: "archivebox.fill").foregroundStyle(.secondary).help("Saved independently of Docker")
                            }
                        }.padding(.vertical, 7).tag(service.reference)
                    }.listStyle(.inset).frame(minWidth: 210, idealWidth: 240, maxWidth: 300)
                    if let service = services.first(where: { $0.reference == selection }) ?? services.first {
                        details(service).id(service.reference)
                    }
                }
            }
            Divider()
            HStack {
                Label("\(library.archived.count) archived · \(storageSize(library.sizeBytes))", systemImage: "archivebox")
                Spacer()
                Label(library.status?.docker.available == true ? "Docker Running" : "Docker Stopped", systemImage: "circle.fill")
                    .foregroundStyle(library.status?.docker.available == true ? .secondary : Color.orange)
                    .help(library.status?.docker.detail ?? "")
            }.font(.caption).foregroundStyle(.secondary).padding(.horizontal, 18).padding(.vertical, 10)
        }
        .task { await library.refresh() }
        .sheet(isPresented: Binding(get: { importPath != nil }, set: { if !$0 { importPath = nil } })) {
            if let importPath { ImageImportView(path: importPath, model: model) }
        }
        .onChange(of: model.jobs.filter { !$0.isActive }.map(\.id)) { _, _ in Task { await library.refresh() } }
        .sheet(item: $helpRequest) { DevlineHelpView(id: $0.id) }
        .confirmationDialog("Remove the archived image?", isPresented: Binding(get: { removing != nil }, set: { if !$0 { removing = nil } }), titleVisibility: .visible, presenting: removing) { service in
            Button("Remove Archive", role: .destructive) {
                Task {
                    _ = await model.mutate([CoreRequest("infra.remove", id: service.reference)], label: "Removing archive", completion: "Removed \(service.name) from the image library.")
                    await library.refresh()
                }
            }
        } message: { service in Text("\(service.name) \(service.version) will no longer be restorable offline from this Mac. Running services and their data are kept.") }
    }

    private func details(_ service: InfrastructureService) -> some View {
        let image = library.image(service.reference)
        return ScrollView {
            VStack(alignment: .leading, spacing: 24) {
                VStack(alignment: .leading, spacing: 10) {
                    Label(service.category, systemImage: service.symbol).font(.callout).foregroundStyle(.secondary)
                    Text(service.name).font(.largeTitle.weight(.semibold))
                    Text(service.description).font(.body).foregroundStyle(.secondary).fixedSize(horizontal: false, vertical: true)
                    Text("\(service.version)  ·  \(service.image.platform)").font(.callout.monospacedDigit()).foregroundStyle(.secondary)
                }
                HStack(spacing: 10) {
                    if image?.archived == true {
                        Button(image?.loaded == true ? "Restore Again" : "Restore to Docker") { start("infra_restore", service) }
                            .buttonStyle(.borderedProminent).disabled(library.status?.docker.available != true || image?.verifiedArchive != true)
                        Button("Verify Archive") { start("infra_verify", service) }
                    } else {
                        Button("Download for Offline Use") { start("infra_prepare", service) }.buttonStyle(.borderedProminent)
                            .disabled(library.status?.docker.available != true)
                    }
                    Menu {
                        Button("Export Image…") { export([service.reference]) }.disabled(image?.archived != true)
                        Button("Remove Archive…", role: .destructive) { removing = service }.disabled(image?.archived != true)
                    } label: { Image(systemName: "ellipsis.circle") }.accessibilityLabel("More actions for \(service.name)")
                }
                VStack(alignment: .leading, spacing: 12) {
                    statusRow("Archived in Devline", detail: archiveDetail(image), ready: image?.verifiedArchive == true)
                    statusRow("Loaded in Docker", detail: image?.loaded == true ? "Available to start in a project" : "Restore the archive when Docker needs it", ready: image?.loaded == true)
                }.padding(16).frame(maxWidth: .infinity, alignment: .leading).background(.quaternary.opacity(0.4), in: RoundedRectangle(cornerRadius: 12))
                if library.status?.docker.available != true {
                    Text("Start Docker Desktop, OrbStack, or another local Docker engine to prepare images and run services. Your archives and documentation remain available while it is stopped.")
                        .font(.callout).foregroundStyle(.secondary)
                }
                Divider()
                VStack(alignment: .leading, spacing: 10) {
                    Text("Use in a project").font(.headline)
                    Text("Add this service when creating a project, or open an existing project and choose Add Services. Devline keeps the version pinned and gives each project its own data volumes.")
                        .font(.callout).foregroundStyle(.secondary)
                    HStack {
                        if let help = service.helpRef {
                            Button("Read Devline Guide") { helpRequest = HelpRequest(id: help) }
                        }
                        if let reference = service.docsRef {
                            Button("Read Offline Reference") { openDocs(reference) }
                        }
                    }
                    if service.docsRef != nil, let coverage = service.docsCoverage {
                        DisclosureGroup("Offline Reference Coverage") {
                            VStack(alignment: .leading, spacing: 6) {
                                Text(coverage)
                                if let omissions = service.docsOmissions {
                                    Text(omissions)
                                }
                            }
                            .font(.callout).foregroundStyle(.secondary)
                        }
                    } else if service.docsRef == nil {
                        Text("Official offline reference unavailable for this release.")
                            .font(.caption).foregroundStyle(.secondary)
                    }
                    if let publisher = service.provenance.docsUrl.flatMap(safeExternalURL) {
                        Link("Publisher Documentation", destination: publisher)
                            .help("Open publisher documentation in your browser")
                    }
                }
                if let receipt = image?.receipt {
                    LabeledContent("Last archive verification", value: Date(timeIntervalSince1970: receipt.verifiedAt).formatted(date: .abbreviated, time: .shortened))
                        .font(.callout).foregroundStyle(.secondary)
                }
                DisclosureGroup("Image Source and Integrity") {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Publisher: \(service.provenance.publisher)")
                        Text("License: \(service.provenance.license)")
                        Text("\(service.image.registry)/\(service.image.repository):\(service.image.tag)")
                        Text(service.image.digest).font(.caption.monospaced())
                        if let url = safeExternalURL(service.provenance.imageUrl) { Link("Publisher’s image page", destination: url) }
                    }.font(.callout).textSelection(.enabled).padding(.top, 10)
                }
            }.padding(28).frame(maxWidth: 700, alignment: .leading).frame(maxWidth: .infinity, alignment: .topLeading)
        }.frame(minWidth: 370, maxWidth: .infinity, maxHeight: .infinity)
    }

    private func statusRow(_ title: String, detail: String, ready: Bool) -> some View {
        HStack(alignment: .top, spacing: 10) {
            Image(systemName: ready ? "checkmark.circle.fill" : "circle.dashed").foregroundStyle(ready ? Color.green : .secondary)
            VStack(alignment: .leading, spacing: 4) { Text(title).font(.callout.weight(.medium)); Text(detail).font(.caption).foregroundStyle(.secondary) }
        }
    }
    private func archiveDetail(_ image: InfrastructureStatus.Image?) -> String {
        guard let image, image.archived else { return "Prepare once while connected to the internet" }
        guard image.verifiedArchive else { return "Verify the saved archive before restoring it" }
        return "\(storageSize(image.receipt?.sizeBytes ?? 0)) · Restorable without a registry"
    }
    private func start(_ kind: String, _ service: InfrastructureService) {
        Task { await model.startJob(CoreRequest("jobs.start", id: service.reference, kind: kind)) }
    }
    private func export(_ images: [String]) {
        let panel = NSSavePanel(); panel.title = "Export Offline Images"; panel.nameFieldStringValue = "Devline-images.devlineimages"
        if panel.runModal() == .OK, let path = panel.url?.path(percentEncoded: false) {
            Task { await model.startJob(CoreRequest("jobs.start", path: path, kind: "infra_export", images: images)) }
        }
    }
    private func importImages() {
        let panel = NSOpenPanel(); panel.title = "Import Offline Images"; panel.prompt = "Import"
        let imageTypes = ["devlineimages", "devlineinfra"].compactMap { UTType(filenameExtension: $0) }
        panel.allowedContentTypes = imageTypes.isEmpty ? [.data] : imageTypes
        if panel.runModal() == .OK, let path = panel.url?.path(percentEncoded: false) {
            importPath = path
        }
    }
}
