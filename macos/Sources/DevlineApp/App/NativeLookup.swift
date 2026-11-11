import AppKit
import Carbon
import OSLog
import SwiftUI

extension Notification.Name {
    static let devlineLookup = Self("Devline.lookup")
    static let devlineShortcutChanged = Self("Devline.shortcutChanged")
}

struct LookupRequest {
    var query = ""
    var path: String?
    var stack: String?
    var message: String?

    init(query: String = "", path: String? = nil, stack: String? = nil, message: String? = nil) {
        self.query = String(query.trimmingCharacters(in: .whitespacesAndNewlines).prefix(4096))
        self.path = path
        self.stack = stack
        self.message = message
    }

    init?(url: URL) {
        guard ["devline", "devkit"].contains(url.scheme?.lowercased() ?? ""), url.host == "search",
              let components = URLComponents(url: url, resolvingAgainstBaseURL: false) else { return nil }
        let items = components.queryItems ?? []
        self.init(query: items.first { $0.name == "q" }?.value ?? "",
                  path: items.first { $0.name == "path" }?.value,
                  stack: items.first { $0.name == "stack" }?.value)
        if let path, !path.hasPrefix("/") { self.path = nil }
    }
}

@MainActor
final class NativeLookup: NSObject, ObservableObject {
    static let shared = NativeLookup()
    @Published private(set) var shortcutError: String?
    private var hotKeys: [EventHotKeyRef] = []
    private var handler: EventHandlerRef?
    private var observer: NSObjectProtocol?
    var revealWorkspace: (() -> Void)?
    static let choices = ["control-option-d", "control-shift-space", "disabled"]
    static func label(_ choice: String) -> String {
        switch choice {
        case "control-shift-space": "⌃⇧Space"
        case "disabled": "Off"
        default: "⌃⌥D"
        }
    }

    func start() {
        NSApp.servicesProvider = self
        NSUpdateDynamicServices()
        var type = EventTypeSpec(eventClass: OSType(kEventClassKeyboard), eventKind: UInt32(kEventHotKeyPressed))
        let installed = InstallEventHandler(GetEventDispatcherTarget(), { _, event, _ in
            guard let event else { return OSStatus(eventNotHandledErr) }
            var key = EventHotKeyID()
            guard GetEventParameter(event, EventParamName(kEventParamDirectObject), EventParamType(typeEventHotKeyID), nil,
                                    MemoryLayout<EventHotKeyID>.size, nil, &key) == noErr,
                  key.signature == 0x446C6E65 else { return OSStatus(eventNotHandledErr) }
            let selection = key.id == 2
            DispatchQueue.main.async { NativeLookup.shared.lookup(selectedText: selection) }
            return noErr
        }, 1, &type, nil, &handler)
        guard installed == noErr else {
            shortcutError = "macOS could not register the lookup handler (\(installed)). Restart Devline to try again."
            return
        }
        observer = NotificationCenter.default.addObserver(forName: .devlineShortcutChanged, object: nil, queue: .main) { _ in
            Task { @MainActor in NativeLookup.shared.registerShortcuts() }
        }
        registerShortcuts()
    }

    func registerShortcuts() {
        hotKeys.forEach { UnregisterEventHotKey($0) }
        hotKeys = []
        shortcutError = nil
        let choice = UserDefaults.standard.string(forKey: "devline.globalShortcut") ?? "control-option-d"
        guard choice != "disabled" else { return }
        let code = choice == "control-shift-space" ? kVK_Space : kVK_ANSI_D
        let modifiers = choice == "control-shift-space" ? controlKey | shiftKey : controlKey | optionKey
        for id in 1...2 {
            var reference: EventHotKeyRef?
            let flags = id == 2 ? modifiers | cmdKey : modifiers
            let status = RegisterEventHotKey(UInt32(code), UInt32(flags), EventHotKeyID(signature: 0x446C6E65, id: UInt32(id)),
                                             GetEventDispatcherTarget(), 0, &reference)
            Logger(subsystem: "io.devline.desktop", category: "lookup").info("Shortcut \(id) registration: \(status)")
            if status == noErr, let reference { hotKeys.append(reference) }
            else { shortcutError = "This shortcut is in use. Choose the other shortcut or turn it off." }
        }
    }

    func lookup(selectedText: Bool) {
        Logger(subsystem: "io.devline.desktop", category: "lookup").info("Global lookup invoked; selection: \(selectedText)")
        var text = ""
        var message: String?
        if selectedText {
            if AXIsProcessTrusted() {
                let system = AXUIElementCreateSystemWide()
                var focused: CFTypeRef?
                if AXUIElementCopyAttributeValue(system, kAXFocusedUIElementAttribute as CFString, &focused) == .success,
                   let focused, CFGetTypeID(focused) == AXUIElementGetTypeID() {
                    var selected: CFTypeRef?
                    AXUIElementCopyAttributeValue(focused as! AXUIElement, kAXSelectedTextAttribute as CFString, &selected)
                    text = selected as? String ?? ""
                }
                if text.isEmpty { message = "No selected text was available. Type a symbol, or use your editor’s Devline command." }
            } else {
                message = "Selected-text lookup needs Accessibility access. Enable it in Devline Settings, or use Services → Look Up in Devline." 
            }
        }
        open(LookupRequest(query: text, message: message))
    }

    func open(_ request: LookupRequest) {
        revealWorkspace?()
        NSApp.activate(ignoringOtherApps: true)
        NSApp.windows.first { $0.identifier?.rawValue == "workspace" || $0.title == "Devline" || $0.title == "Docs" }?.makeKeyAndOrderFront(nil)
        DispatchQueue.main.async { NotificationCenter.default.post(name: .devlineLookup, object: request) }
    }

    @objc func lookUpInDevline(_ pasteboard: NSPasteboard, userData: String?, error: AutoreleasingUnsafeMutablePointer<NSString>) {
        guard let selection = pasteboard.string(forType: .string), !selection.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else {
            error.pointee = "Select a symbol or phrase to look up."
            return
        }
        open(LookupRequest(query: selection))
    }
}

@MainActor
final class DevlineAppDelegate: NSObject, NSApplicationDelegate {
    func applicationDidFinishLaunching(_ notification: Notification) { NativeLookup.shared.start() }
    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool { false }
    func applicationShouldHandleReopen(_ sender: NSApplication, hasVisibleWindows flag: Bool) -> Bool { true }
}

struct LookupSettings: View {
    @Preference("devline.globalShortcut") private var shortcut = "control-option-d"
    @ObservedObject private var lookup = NativeLookup.shared
    var body: some View {
        Section("Lookup from Any App") {
            Picker("Global lookup", selection: $shortcut) {
                ForEach(NativeLookup.choices, id: \.self) { choice in Text(NativeLookup.label(choice)).tag(choice) }
            }.onChange(of: shortcut) { _, _ in NotificationCenter.default.post(name: .devlineShortcutChanged, object: nil) }
            if shortcut != "disabled" {
                LabeledContent("Look up selected text", value: "Add ⌘ to \(NativeLookup.label(shortcut))")
            }
            if let error = lookup.shortcutError { Text(error).foregroundStyle(.orange).font(.caption) }
            Button("Enable Selected-Text Access…") {
                let options = [kAXTrustedCheckOptionPrompt.takeUnretainedValue() as String: true] as CFDictionary
                _ = AXIsProcessTrustedWithOptions(options)
            }
            Text("You can also select text in an app and choose Services → Look Up in Devline. The Service does not require Accessibility access.")
                .font(.caption).foregroundStyle(.secondary)
            Link("Editor integration instructions", destination: URL(string: "https://github.com/fpl0/devline/tree/main/integrations")!)
        }
    }
}
