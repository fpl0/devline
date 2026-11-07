import AppKit
import Sparkle
import SwiftUI

@MainActor
final class AppUpdates: ObservableObject {
    static let shared = AppUpdates()
    @Published private(set) var canCheck = false
    @Published private(set) var automaticChecks = false
    let configured: Bool
    var usesSignedUpdates: Bool { configured }
    private let controller: SPUStandardUpdaterController

    private init() {
        // A public key alone does not mean this release has a signed appcast.
        configured = Bundle.main.object(forInfoDictionaryKey: "DevlineSignedUpdatesEnabled") as? Bool == true
            && Bundle.main.object(forInfoDictionaryKey: "SUPublicEDKey") as? String != nil
            && Bundle.main.object(forInfoDictionaryKey: "SUFeedURL") as? String != nil
        controller = SPUStandardUpdaterController(startingUpdater: configured, updaterDelegate: nil, userDriverDelegate: nil)
        controller.updater.publisher(for: \.canCheckForUpdates).assign(to: &$canCheck)
        controller.updater.publisher(for: \.automaticallyChecksForUpdates).assign(to: &$automaticChecks)
    }

    func check() { if configured { controller.checkForUpdates(nil) } }
    func setAutomaticChecks(_ enabled: Bool) { if configured { controller.updater.automaticallyChecksForUpdates = enabled } }
}

struct AppUpdatesSettings: View {
    @ObservedObject private var updates = AppUpdates.shared
    @State private var release: UpdateItem?
    @State private var checkedAt: TimeInterval?
    @State private var checking = false
    @State private var error: String?
    private let releases = URL(string: "https://github.com/fpl0/devline/releases/latest")!

    var body: some View {
        Section("App Updates") {
            if updates.usesSignedUpdates {
                Toggle("Check automatically", isOn: Binding(get: { updates.automaticChecks }, set: updates.setAutomaticChecks))
                Button("Check for App Updates…", action: updates.check).disabled(!updates.canCheck)
                Text("App updates are authenticated before installation. Automatic checks are optional; your offline library always remains available.")
                    .font(.caption).foregroundStyle(.secondary)
            } else {
                LabeledContent("Installed version", value: Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? "Development build")
                if let release {
                    Label(release.state.label, systemImage: release.state.icon)
                        .foregroundStyle(release.state == .error ? Color.orange : .secondary)
                    Text(release.detail).font(.callout).foregroundStyle(.secondary).textSelection(.enabled)
                }
                HStack {
                    Button("Check App Release") { Task { await load(online: true) } }.disabled(checking)
                    if checking { ProgressView().controlSize(.small) }
                    Spacer()
                    Link(release?.state == .available ? "Download New Release" : "View Releases", destination: release?.externalURL ?? releases)
                }
                if let checkedAt {
                    Text("Last checked \(Date(timeIntervalSince1970: checkedAt).formatted(date: .abbreviated, time: .shortened))")
                        .font(.caption).foregroundStyle(.secondary)
                }
                if let error { Text(error).font(.callout).foregroundStyle(.orange).textSelection(.enabled) }
                Text("This build uses direct downloads. Download and replace the app to update it; your prepared library stays on this Mac. Automatic installation is not enabled for this release.")
                    .font(.caption).foregroundStyle(.secondary)
            }
        }.task { if !updates.usesSignedUpdates { await load(online: false) } }
    }

    private func load(online: Bool) async {
        guard !checking else { return }
        checking = true; error = nil
        defer { checking = false }
        do {
            let result: UpdateSnapshot = try await Core.request(CoreRequest(online ? "updates.app.check" : "updates.app.status"))
            release = result.items.first { $0.kind == .app }
            checkedAt = result.checkedAt
        } catch is CancellationError {} catch { self.error = error.localizedDescription }
    }
}
