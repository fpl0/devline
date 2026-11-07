import AppKit
import Sparkle
import SwiftUI

@MainActor
final class AppUpdates: ObservableObject {
    static let shared = AppUpdates()
    @Published private(set) var canCheck = false
    @Published private(set) var automaticChecks = false
    let configured: Bool
    private let controller: SPUStandardUpdaterController

    private init() {
        configured = Bundle.main.object(forInfoDictionaryKey: "SUPublicEDKey") as? String != nil
        controller = SPUStandardUpdaterController(startingUpdater: configured, updaterDelegate: nil, userDriverDelegate: nil)
        controller.updater.publisher(for: \.canCheckForUpdates).assign(to: &$canCheck)
        controller.updater.publisher(for: \.automaticallyChecksForUpdates).assign(to: &$automaticChecks)
    }

    func check() { controller.checkForUpdates(nil) }
    func setAutomaticChecks(_ enabled: Bool) { controller.updater.automaticallyChecksForUpdates = enabled }
}

struct AppUpdatesSettings: View {
    @ObservedObject private var updates = AppUpdates.shared
    var body: some View {
        Section("App Updates") {
            Toggle("Check automatically", isOn: Binding(get: { updates.automaticChecks }, set: updates.setAutomaticChecks))
                .disabled(!updates.configured)
            Button("Check for App Updates…", action: updates.check).disabled(!updates.canCheck)
            Text("App updates are authenticated before installation. Automatic checks are optional; your offline library always remains available.")
                .font(.caption).foregroundStyle(.secondary)
        }
    }
}
