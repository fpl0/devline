import AppKit
import FocusCore
import Foundation
import Observation
import SwiftUI
import UniformTypeIdentifiers

@MainActor @Observable
final class FocusStore {
    private let file: FocusFile
    @ObservationIgnored private var ticker: Task<Void, Never>?
    var state: SessionJournal?
    var failure: String?
    var now = Date()
    var notice = "Choose one thing to focus on."
    var completionPending = false
    init() {
        let root =
            ProcessInfo.processInfo.environment["FOCUS_DATA_DIR"].map { URL(fileURLWithPath: $0) }
            ?? URL.applicationSupportDirectory.appendingPathComponent(
                Bundle.main.bundleIdentifier ?? "local.devline.focus-sessions")
        file = FocusFile(url: root.appendingPathComponent("sessions.json"))
        reload()
        ticker = Task { [weak self] in
            while !Task.isCancelled {
                do { try await Task.sleep(for: .seconds(1)) } catch { return }
                guard let self else { return }
                self.tick()
            }
        }
    }
    deinit { ticker?.cancel() }
    func reload() {
        do {
            state = try file.load()
            failure = nil
            tick()
        } catch {
            state = nil
            failure = error.localizedDescription
        }
    }
    func change(_ operation: (inout SessionJournal) throws -> Void) {
        guard let before = state else { return }
        do {
            var next = before
            try operation(&next)
            guard next != before else { return }
            try file.save(next, expecting: before)
            state = next
            failure = nil
            if next.active != nil { completionPending = false }
        } catch { failure = error.localizedDescription }
    }
    func tick() {
        now = Date()
        guard let active = state?.active, !active.isPaused, active.remaining(at: now) == 0 else {
            return
        }
        change { _ = $0.advance(to: now) }
        if state?.active == nil {
            notice =
                active.kind == .focus
                ? "Session complete. Take a break." : "Break complete. Ready for another session?"
            completionPending = true
            NSSound(named: NSSound.Name("Glass"))?.play()
        }
    }
    var clock: String {
        let seconds = Int(
            ceil(state?.active?.remaining(at: now) ?? Double((state?.focusMinutes ?? 25) * 60)))
        return String(format: "%02d:%02d", seconds / 60, seconds % 60)
    }
    func export() {
        guard let state else { return }
        let panel = NSSavePanel()
        panel.allowedContentTypes = [.commaSeparatedText]
        panel.nameFieldStringValue = "focus-sessions.csv"
        guard panel.runModal() == .OK, let url = panel.url else { return }
        do { try state.csv.write(to: url, atomically: true, encoding: .utf8) } catch {
            failure = error.localizedDescription
        }
    }
}

@main
struct FocusSessionsApp: App {
    @State private var store = FocusStore()
    var body: some Scene {
        Window("Focus Sessions", id: "sessions") {
            SessionView(store: store, compact: false)
        }.defaultSize(width: 620, height: 680)
        MenuBarExtra {
            SessionView(store: store, compact: true)
        } label: {
            Label(
                store.state?.active == nil
                    ? (store.completionPending ? "Done" : "Focus") : store.clock,
                systemImage: "timer")
        }.menuBarExtraStyle(.window)
    }
}

private struct SessionView: View {
    let store: FocusStore
    let compact: Bool
    @Environment(\.openWindow) private var openWindow
    @State private var label = ""
    @State private var pendingStop = false

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 18) {
                HStack {
                    Label("Focus Sessions", systemImage: "timer").font(.title2.bold())
                    Spacer()
                    if compact {
                        Button {
                            openWindow(id: "sessions")
                            NSApp.activate(ignoringOtherApps: true)
                        } label: {
                            Image(systemName: "arrow.up.left.and.arrow.down.right")
                        }.help("Open session history")
                    }
                }
                if let state = store.state {
                    timerCard(state)
                    if !compact { history(state) }
                } else {
                    ContentUnavailableView(
                        "Unable to Load Sessions", systemImage: "exclamationmark.triangle")
                    Button("Reload", action: store.reload)
                }
                if let failure = store.failure {
                    Text(failure).foregroundStyle(.red).textSelection(.enabled)
                    Button("Reload Saved Sessions", action: store.reload)
                }
                if compact { Button("Quit Focus Sessions") { NSApp.terminate(nil) }.font(.caption) }
            }.padding(24)
        }
        .frame(
            minWidth: compact ? 340 : 500, maxWidth: compact ? 380 : .infinity,
            minHeight: compact ? 280 : 500
        )
        .confirmationDialog("Stop this session?", isPresented: $pendingStop) {
            Button("Stop and Save Elapsed Focus", role: .destructive) {
                store.change { $0.stop(at: Date()) }
            }
        } message: {
            Text("Time already focused will remain in your history.")
        }
    }

    private func timerCard(_ state: SessionJournal) -> some View {
        VStack(alignment: .leading, spacing: 16) {
            if let active = state.active {
                Text(active.label).font(.headline)
                Text(
                    active.isPaused
                        ? "Paused" : (active.kind == .focus ? "Focusing" : "Taking a break")
                )
                .font(.subheadline).foregroundStyle(.secondary)
                Text(store.clock).font(
                    .system(size: compact ? 48 : 72, weight: .light, design: .rounded)
                ).monospacedDigit()
                ProgressView(
                    value: active.duration - active.remaining(at: store.now), total: active.duration
                )
                HStack {
                    Button(active.isPaused ? "Resume" : "Pause") {
                        store.change { state in
                            if active.isPaused {
                                try state.resume(at: Date())
                            } else {
                                try state.pause(at: Date())
                            }
                        }
                    }.buttonStyle(.borderedProminent)
                    Button("Stop…") { pendingStop = true }
                }
            } else {
                Text(store.notice).foregroundStyle(.secondary)
                TextField("What will you focus on?", text: $label).textFieldStyle(.roundedBorder)
                    .onSubmit { startFocus() }
                Stepper(
                    "Focus: \(state.focusMinutes) min",
                    value: Binding(
                        get: { state.focusMinutes },
                        set: { value in
                            store.change {
                                try $0.configure(
                                    focusMinutes: value, breakMinutes: state.breakMinutes)
                            }
                        }
                    ), in: 1...120)
                Stepper(
                    "Break: \(state.breakMinutes) min",
                    value: Binding(
                        get: { state.breakMinutes },
                        set: { value in
                            store.change {
                                try $0.configure(
                                    focusMinutes: state.focusMinutes, breakMinutes: value)
                            }
                        }
                    ), in: 1...30)
                HStack {
                    Button("Start Focus", action: startFocus).buttonStyle(.borderedProminent)
                        .disabled(
                            label.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
                                || label.count > 120)
                    Button("Take a Break") {
                        store.change { try $0.start(label: "", kind: .rest, at: Date()) }
                    }
                }
            }
            Divider()
            Text("Today: \(state.totalSeconds(on: store.now) / 60) min focused")
                .font(.subheadline).foregroundStyle(.secondary)
        }.padding(20).background(.quaternary.opacity(0.35), in: RoundedRectangle(cornerRadius: 16))
    }

    private func startFocus() { store.change { try $0.start(label: label, at: Date()) } }

    private func history(_ state: SessionJournal) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("Session History").font(.headline)
                Spacer()
                Button("Export CSV…", action: store.export).disabled(state.history.isEmpty)
            }
            if state.history.isEmpty {
                Text("Completed and stopped focus sessions appear here. Breaks are excluded.")
                    .foregroundStyle(.secondary)
            }
            ForEach(state.history) { session in
                HStack(alignment: .top) {
                    Image(systemName: session.completed ? "checkmark.circle.fill" : "stop.circle")
                        .foregroundStyle(session.completed ? .green : .secondary)
                    VStack(alignment: .leading) {
                        Text(session.label).font(.headline)
                        Text(session.finishedAt, format: .dateTime.day().month().hour().minute())
                            .font(.caption).foregroundStyle(.secondary)
                    }
                    Spacer()
                    Text("\(session.focusedSeconds / 60)m \(session.focusedSeconds % 60)s")
                        .monospacedDigit()
                }.padding(.vertical, 5)
            }
        }
    }
}
