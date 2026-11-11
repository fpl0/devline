import SwiftUI

/// A page that stays loaded after its first visit, so switching back to it is immediate.
///
/// While it is off screen it is left completely alone: it is not rebuilt when the window's state
/// changes, and nothing in its environment is touched, because either would make every switch
/// pay for every page. It catches up the moment it is shown again.
struct KeptPage<Content: View>: View, Equatable {
    let shown: Bool
    @ViewBuilder let content: () -> Content

    nonisolated static func == (old: Self, new: Self) -> Bool { !old.shown && !new.shown }

    var body: some View {
        // Opacity alone takes a hidden page's controls out of the window, so they cannot be
        // clicked or tabbed to. Turning the page off with `disabled` would do the same, but it
        // changes the environment of every view on the page, which costs far more.
        content()
            .opacity(shown ? 1 : 0)
            .allowsHitTesting(shown)
            .accessibilityHidden(!shown)
    }
}

extension View {
    /// Runs when a page that stayed loaded comes back on screen, to bring it up to date quietly.
    func onPageShown(_ visible: Bool, _ action: @escaping () async -> Void) -> some View {
        onChange(of: visible) { _, now in
            if now { Task { await action() } }
        }
    }
}
