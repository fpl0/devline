import SwiftUI

/// Devline's content surfaces: one quiet fill, one hairline, continuous corners. Glass is left to
/// the system's own navigation layer (sidebar, toolbar) and to things that float above content.
private struct Surface: ViewModifier {
    let radius: CGFloat
    let inset: Bool

    func body(content: Content) -> some View {
        let shape = RoundedRectangle(cornerRadius: radius, style: .continuous)
        content
            .background(inset ? AnyShapeStyle(.background.opacity(0.55)) : AnyShapeStyle(Color.primary.opacity(0.045)), in: shape)
            .overlay { shape.strokeBorder(Color.primary.opacity(inset ? 0.05 : 0.07), lineWidth: 1) }
    }
}

extension View {
    /// A card that groups related content on a page.
    func surface(radius: CGFloat = 14) -> some View { modifier(Surface(radius: radius, inset: false)) }

    /// A panel nested inside a card, or a well for output.
    func insetSurface(radius: CGFloat = 10) -> some View { modifier(Surface(radius: radius, inset: true)) }

    /// The background for something that floats above content, such as a status message.
    @ViewBuilder func floatingCapsule() -> some View {
        if #available(macOS 26.0, *) {
            glassEffect()
        } else {
            background(.regularMaterial, in: Capsule())
                .overlay { Capsule().strokeBorder(Color.primary.opacity(0.1), lineWidth: 1) }
                .shadow(color: .black.opacity(0.18), radius: 12, y: 4)
        }
    }
}

/// A short message that floats over the bottom of the window: work in progress, or its result.
struct StatusToast: View {
    let text: String
    let working: Bool
    var dismiss: (() -> Void)?

    var body: some View {
        HStack(spacing: 9) {
            if working { ProgressView().controlSize(.small) }
            else { Image(systemName: "checkmark.circle.fill").foregroundStyle(.green) }
            Text(text).font(.callout).lineLimit(2)
            if let dismiss {
                Button(action: dismiss) { Image(systemName: "xmark").font(.caption.weight(.semibold)) }
                    .buttonStyle(.plain).foregroundStyle(.secondary)
                    .help("Dismiss").accessibilityLabel("Dismiss status")
            }
        }
        .padding(.horizontal, 16).padding(.vertical, 10)
        .floatingCapsule()
        .frame(maxWidth: 560)
        .accessibilityElement(children: .combine)
    }
}
