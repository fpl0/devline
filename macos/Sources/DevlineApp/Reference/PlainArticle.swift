import AppKit
import SwiftUI

struct PlainArticle: NSViewRepresentable {
    let article: ReaderArticle
    let horizontalInset: CGFloat
    let scale: Double
    let request: ReaderRequest?
    let report: (ReaderRequest, Bool) -> Void
    let reportPosition: (Double) -> Void

    func makeCoordinator() -> Coordinator { Coordinator() }

    func makeNSView(context: Context) -> NSScrollView {
        let scrollView = NSTextView.scrollableTextView()
        scrollView.drawsBackground = false
        guard let textView = scrollView.documentView as? NSTextView else { return scrollView }
        textView.drawsBackground = false
        textView.isEditable = false
        textView.isSelectable = true
        textView.textContainerInset = NSSize(width: horizontalInset, height: 20)
        textView.setAccessibilityLabel("Document contents")
        textView.textStorage?.setAttributedString(scaledContents())
        context.coordinator.attach(scrollView, report: reportPosition)
        return scrollView
    }

    func updateNSView(_ scrollView: NSScrollView, context: Context) {
        guard let textView = scrollView.documentView as? NSTextView else { return }
        context.coordinator.reportPosition = reportPosition
        if context.coordinator.scale != scale {
            context.coordinator.scale = scale
            textView.textStorage?.setAttributedString(scaledContents())
        }
        if textView.textContainerInset.width != horizontalInset {
            textView.textContainerInset = NSSize(width: horizontalInset, height: 20)
        }
        guard context.coordinator.last != request else { return }
        let last = context.coordinator.last
        context.coordinator.last = request
        guard let request else {
            textView.setSelectedRange(NSRange(location: textView.selectedRange().location, length: 0))
            return
        }

        let coordinator = context.coordinator
        DispatchQueue.main.async {
            guard coordinator.last == request else { return }
            scrollView.layoutSubtreeIfNeeded()
            apply(request, last: last, to: textView, in: scrollView)
        }
    }

    private func apply(_ request: ReaderRequest, last: ReaderRequest?, to textView: NSTextView,
                       in scrollView: NSScrollView) {
        switch request.action {
        case .focus(let text):
            apply(ReaderRequest(action: .find(text: text, backwards: false, fromStart: true)), last: last, to: textView, in: scrollView)
        case .restore(let fraction, _, _):
            let height = max(0, textView.frame.height - scrollView.contentView.bounds.height)
            scrollView.contentView.scroll(to: NSPoint(x: 0, y: height * min(1, max(0, fraction))))
            scrollView.reflectScrolledClipView(scrollView.contentView)
        case .jump(let section):
            guard case .text(let range) = section.target else { return }
            textView.setSelectedRange(NSRange(location: range.location, length: 0))
            scrollTo(range, textView: textView, scrollView: scrollView)
        case .find(let text, let backwards, let fromStart):
            let contents = textView.string as NSString
            let whole = NSRange(location: 0, length: contents.length)
            let selection = textView.selectedRange()
            let next = min(NSMaxRange(selection), contents.length)
            var options: NSString.CompareOptions = [.caseInsensitive, .diacriticInsensitive]
            if backwards { options.insert(.backwards) }
            let range: NSRange
            if fromStart || last == nil { range = whole }
            else if backwards { range = NSRange(location: 0, length: selection.location) }
            else { range = NSRange(location: next, length: contents.length - next) }
            var found = contents.range(of: text, options: options, range: range)
            if found.location == NSNotFound && range != whole {
                found = contents.range(of: text, options: options, range: whole)
            }
            if found.location != NSNotFound {
                textView.setSelectedRange(found)
                if fromStart { scrollTo(found, textView: textView, scrollView: scrollView) }
                else { textView.scrollRangeToVisible(found) }
                textView.showFindIndicator(for: found)
            }
            report(request, found.location != NSNotFound)
        }
    }

    private func scrollTo(_ range: NSRange, textView: NSTextView, scrollView: NSScrollView) {
        textView.scrollRangeToVisible(range)
        guard let layout = textView.layoutManager, let container = textView.textContainer else { return }
        layout.ensureLayout(forCharacterRange: range)
        let glyphs = layout.glyphRange(forCharacterRange: range, actualCharacterRange: nil)
        let rect = layout.boundingRect(forGlyphRange: glyphs, in: container)
        let maxY = max(0, textView.frame.height - scrollView.contentView.bounds.height)
        let y = min(maxY, max(0, rect.minY + textView.textContainerOrigin.y - 25))
        scrollView.contentView.scroll(to: NSPoint(x: 0, y: y))
        scrollView.reflectScrolledClipView(scrollView.contentView)
    }

    private func scaledContents() -> NSAttributedString {
        let contents = NSMutableAttributedString(attributedString: article.contents)
        contents.enumerateAttribute(.font, in: NSRange(location: 0, length: contents.length)) { value, range, _ in
            if let font = value as? NSFont {
                contents.addAttribute(.font, value: NSFontManager.shared.convert(font, toSize: font.pointSize * scale), range: range)
            }
        }
        return contents
    }

    final class Coordinator {
        var last: ReaderRequest?
        var scale = 1.0
        var reportPosition: ((Double) -> Void)?
        private var observer: NSObjectProtocol?
        func attach(_ view: NSScrollView, report: @escaping (Double) -> Void) {
            reportPosition = report
            view.contentView.postsBoundsChangedNotifications = true
            observer = NotificationCenter.default.addObserver(forName: NSView.boundsDidChangeNotification,
                                                               object: view.contentView, queue: .main) { [weak self, weak view] _ in
                guard let view, let content = view.documentView else { return }
                let maximum = max(0, content.frame.height - view.contentView.bounds.height)
                self?.reportPosition?(maximum > 0 ? view.contentView.bounds.minY / maximum : 0)
            }
        }
        deinit { if let observer { NotificationCenter.default.removeObserver(observer) } }
    }
}
