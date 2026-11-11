import SwiftUI

/// Tells the views that show one saved setting when it changes.
@MainActor
final class PreferenceBox: ObservableObject {
    private static var boxes: [String: PreferenceBox] = [:]

    static func box(_ key: String) -> PreferenceBox {
        if let box = boxes[key] { return box }
        let box = PreferenceBox(key: key)
        boxes[key] = box
        return box
    }

    let key: String
    private init(key: String) { self.key = key }

    func set(_ value: Any?) {
        objectWillChange.send()
        UserDefaults.standard.set(value, forKey: key)
    }
}

/// A saved setting.
///
/// `@AppStorage` cannot watch a single key whose name contains a dot, so for those it wakes on
/// every write to the defaults. Devline's keys all contain one, which made each write (the current
/// page, the open project, a reading position) rebuild the whole window. A view that uses this
/// wrapper is rebuilt only when its own setting changes.
@MainActor
@propertyWrapper
struct Preference<Value>: DynamicProperty {
    @ObservedObject private var box: PreferenceBox
    private let fallback: Value

    init(wrappedValue: Value, _ key: String) {
        box = PreferenceBox.box(key)
        fallback = wrappedValue
    }

    var wrappedValue: Value {
        get { UserDefaults.standard.object(forKey: box.key) as? Value ?? fallback }
        nonmutating set { box.set(newValue) }
    }

    var projectedValue: Binding<Value> {
        Binding(get: { wrappedValue }, set: { wrappedValue = $0 })
    }
}

/// Applies the appearance chosen in Settings.
private struct PreferredAppearance: ViewModifier {
    @Preference("devline.appearance") private var appearance = "system"

    func body(content: Content) -> some View {
        content.preferredColorScheme(appearance == "system" ? nil : appearance == "light" ? .light : .dark)
    }
}

extension View {
    func preferredAppearance() -> some View { modifier(PreferredAppearance()) }
}
