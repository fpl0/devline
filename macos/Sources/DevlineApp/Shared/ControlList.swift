import SwiftUI

// These short catalogs contain controls and variable-height rows. Lay out the
// whole catalog so interacting with a control does not change row estimates.
struct ControlList<Content: View>: View {
    var horizontalPadding: CGFloat = 18
    @ViewBuilder var content: Content

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 0) {
                content
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.horizontal, horizontalPadding)
            .padding(.bottom, 8)
        }
    }
}

struct ControlListSection<Content: View>: View {
    let title: String
    @ViewBuilder var content: Content

    init(_ title: String, @ViewBuilder content: () -> Content) {
        self.title = title
        self.content = content()
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Text(title)
                .font(.subheadline.weight(.semibold)).foregroundStyle(.secondary)
                .accessibilityAddTraits(.isHeader)
                .padding(.top, 14).padding(.bottom, 7)
            content
        }
    }
}

struct ControlListRow<Content: View>: View {
    @ViewBuilder var content: Content

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Divider()
            content.padding(.vertical, 7)
        }
    }
}
