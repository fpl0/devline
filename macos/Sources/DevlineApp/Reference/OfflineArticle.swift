import AppKit
import SwiftUI
import WebKit

struct OfflineArticle: NSViewRepresentable {
    let html: String
    let title: String
    let sourceURL: String
    let packID: String
    let scale: Double
    let request: ReaderRequest?
    let report: (ReaderRequest, Bool) -> Void
    let reportSections: ([ReaderSection]) -> Void
    var linkRevision = ""
    var linkStatuses: (([String]) async throws -> [DocLinkStatus])?
    let navigate: (String, DocLinkStatus?) -> Void
    let reportPosition: (Double, String?, Double?, Bool) -> Void

    func makeCoordinator() -> Coordinator { Coordinator() }

    func makeNSView(context: Context) -> WKWebView {
        let configuration = WKWebViewConfiguration()
        configuration.defaultWebpagePreferences.allowsContentJavaScript = false
        configuration.websiteDataStore = .nonPersistent()
        if let source = Coordinator.highlightingScript {
            configuration.userContentController.addUserScript(WKUserScript(
                source: source, injectionTime: .atDocumentEnd, forMainFrameOnly: true, in: Coordinator.readerWorld
            ))
        }
        configuration.userContentController.add(context.coordinator, contentWorld: Coordinator.readerWorld, name: "readerAction")
        let view = WKWebView(frame: .zero, configuration: configuration)
        view.navigationDelegate = context.coordinator
        view.setAccessibilityLabel("Document contents")
        return view
    }

    func updateNSView(_ view: WKWebView, context: Context) {
        let coordinator = context.coordinator
        coordinator.request = request
        coordinator.report = report
        coordinator.reportSections = reportSections
        coordinator.linkStatuses = linkStatuses
        coordinator.navigate = navigate
        coordinator.reportPosition = reportPosition
        coordinator.sourceURL = sourceURL
        let packChanged = coordinator.packID != packID
        coordinator.packID = packID
        let revisionChanged = coordinator.linkRevision != linkRevision
        coordinator.linkRevision = linkRevision
        if view.pageZoom != scale { view.pageZoom = scale }
        if coordinator.lastHTML != html || coordinator.lastTitle != title || packChanged {
            coordinator.load(page, html: html, title: title, in: view)
        } else {
            coordinator.applyRequest(in: view)
            if revisionChanged { coordinator.refreshLinks(in: view) }
        }
    }

    static func dismantleNSView(_ view: WKWebView, coordinator: Coordinator) {
        coordinator.cancelLinkCheck()
        view.configuration.userContentController.removeScriptMessageHandler(forName: "readerAction", contentWorld: Coordinator.readerWorld)
        view.navigationDelegate = nil
    }

    private var page: String {
        let hasTitle = html.range(of: #"(?i)<h1\b"#, options: .regularExpression) != nil
        let escapedTitle = title.replacingOccurrences(of: "&", with: "&amp;")
            .replacingOccurrences(of: "<", with: "&lt;").replacingOccurrences(of: ">", with: "&gt;")
        let heading = hasTitle ? "" : "<h1 class=\"reader-title\">\(escapedTitle)</h1>"
        return """
        <!doctype html><html lang="en"><meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1">
        <meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src data:; style-src 'unsafe-inline'; base-uri 'none'; form-action 'none'">
        <style>
        :root { color-scheme: light dark; --line: color-mix(in srgb, CanvasText 18%, Canvas); --soft: color-mix(in srgb, CanvasText 6%, Canvas); }
        * { box-sizing: border-box; }
        body { margin: 0; background: Canvas; color: CanvasText; font: 14px/1.65 -apple-system, system-ui, sans-serif; }
        main { max-inline-size: 800px; margin-inline: auto; padding: 24px 28px 48px; overflow-wrap: anywhere; }
        h1, h2, h3, h4, h5, h6, dt, section[id] { scroll-margin-block-start: 24px; }
        h1,h2,h3,h4,h5,h6 { margin: 28px 0 11px; line-height: 1.3; }
        h1 { font-size: 28px; font-weight: 650; } h2 { font-size: 21px; font-weight: 650; border-block-start: 1px solid var(--line); padding-block-start: 20px; }
        h3 { font-size: 17px; font-weight: 650; } h4,h5,h6 { font-size: 15px; font-weight: 650; }
        main > :first-child { margin-block-start: 0; }
        p,ul,ol,dl,blockquote,pre,.table-scroll,.code-region { margin: 0 0 15px; }
        ul,ol { padding-inline-start: 1.6em; } li + li { margin-block-start: 5px; } li > p { margin-block-end: 5px; }
        li > ul,li > ol { margin-block: 5px 0; }
        dt { font-weight: 600; margin-block: 18px 8px; } dt[id] { font: 600 13px/1.6 ui-monospace, Menlo, monospace; background: var(--soft); padding: 10px 12px; border-radius: 5px; }
        dd { margin-inline: 0; padding-inline-start: 14px; } dl.field-list { display: grid; grid-template-columns: auto 1fr; gap: 8px 14px; }
        blockquote,.admonition,.note,.warning,.tip,.important,.caution { padding: 10px 14px; border-inline-start: 3px solid var(--line); background: var(--soft); border-radius: 3px; margin-block: 16px; }
        .admonition-title { font-weight: 650; } .warning,.caution { border-color: #b57913; }
        code,kbd,samp { font: 0.91em ui-monospace, Menlo, monospace; background: var(--soft); padding: 1px 4px; border-radius: 4px; white-space: break-spaces; }
        pre,.table-scroll { max-inline-size: 100%; overflow-x: auto; overscroll-behavior-inline: contain; }
        pre { background: var(--soft); border: 1px solid var(--line); border-radius: 7px; padding: 35px 14px 14px; font: 12.5px/1.58 ui-monospace, Menlo, monospace; tab-size: 4; white-space: pre; overflow-wrap: normal; }
        pre code { font: inherit; background: none; padding: 0; white-space: inherit; }
        .code-region { position: relative; } .code-region pre { margin: 0; }
        .code-copy { position: absolute; inset-block-start: 7px; inset-inline-end: 9px; border: 0; border-radius: 4px; padding: 3px 7px; font: 11px -apple-system, system-ui; background: Canvas; color: CanvasText; cursor: pointer; }
        .code-copy:hover { background: var(--line); }
        table { min-inline-size: 100%; border-collapse: collapse; font-size: 13px; }
        th,td { padding: 8px 11px; text-align: start; vertical-align: top; border-block-end: 1px solid var(--line); }
        th { background: var(--soft); font-weight: 650; } td code,th code { white-space: nowrap; }
        hr { border: 0; border-block-start: 1px solid var(--line); margin-block: 24px; }
        img { max-inline-size: 100%; height: auto; } figure { margin-inline: 0; } figcaption { font-size: 12px; opacity: .8; }
        a { color: LinkText; text-underline-offset: 2px; } a:hover { text-decoration-thickness: 2px; }
        a[data-link-status="online"]::after { content: " ↗ Online"; font: .72em -apple-system, system-ui; white-space: nowrap; }
        a:is([data-link-status="missingDocument"],[data-link-status="missingSection"],[data-link-status="unavailable"])::after {
            content: " ⓘ"; font: .8em -apple-system, system-ui; white-space: nowrap;
        }
        summary { cursor: pointer; font-weight: 600; padding-block: 7px; } details > :not(summary) { margin-inline-start: 12px; }
        .reference-target { outline: 2px solid AccentColor; outline-offset: 3px; border-radius: 3px; }
        :focus-visible { outline: 2px solid -webkit-focus-ring-color; outline-offset: 2px; }
        pre[data-devline-highlighted] {
            --syntax-keyword: #792b88; --syntax-string: #8b4d16; --syntax-comment: #58616c;
            --syntax-literal: #a03c50; --syntax-name: #155da4; --syntax-type: #006657;
        }
        @media (prefers-color-scheme: dark) {
            pre[data-devline-highlighted] {
                --syntax-keyword: #e5a9ee; --syntax-string: #efbd8d; --syntax-comment: #a6afbb;
                --syntax-literal: #ffa3b1; --syntax-name: #92c3ff; --syntax-type: #8cdbc5;
            }
        }
        pre[data-devline-highlighted] :is(.hljs-keyword,.hljs-selector-tag,.hljs-meta) { color: var(--syntax-keyword); }
        pre[data-devline-highlighted] :is(.hljs-string,.hljs-regexp,.hljs-addition) { color: var(--syntax-string); }
        pre[data-devline-highlighted] :is(.hljs-comment,.hljs-quote) { color: var(--syntax-comment); }
        pre[data-devline-highlighted] :is(.hljs-number,.hljs-literal,.hljs-symbol,.hljs-deletion) { color: var(--syntax-literal); }
        pre[data-devline-highlighted] :is(.hljs-title,.hljs-name,.hljs-section,.hljs-function) { color: var(--syntax-name); }
        pre[data-devline-highlighted] :is(.hljs-type,.hljs-built_in,.hljs-attr,.hljs-attribute,.hljs-variable) { color: var(--syntax-type); }
        pre[data-devline-highlighted] .hljs-subst { color: CanvasText; }
        </style><body><main id="devline-reader">\(heading)\(html)</main></body></html>
        """
    }

    final class Coordinator: NSObject, WKNavigationDelegate, WKScriptMessageHandler {
        static let readerWorld = WKContentWorld.world(name: "DevlineArticle")
        static let highlightingScript: String? = {
            let bundle = Bundle.devlineResources
            let scripts = ["highlight.min", "code-highlighting"].compactMap { name in
                bundle.url(forResource: name, withExtension: "js").flatMap {
                    try? String(contentsOf: $0, encoding: .utf8)
                }
            }
            return scripts.count == 2 ? scripts.joined(separator: "\n") : nil
        }()
        private(set) var lastHTML: String?
        private(set) var lastTitle: String?
        var sourceURL = ""
        var packID = ""
        var request: ReaderRequest?
        var report: ((ReaderRequest, Bool) -> Void)?
        var reportSections: (([ReaderSection]) -> Void)?
        var linkStatuses: (([String]) async throws -> [DocLinkStatus])?
        var linkRevision = ""
        var navigate: ((String, DocLinkStatus?) -> Void)?
        var reportPosition: ((Double, String?, Double?, Bool) -> Void)?
        private var navigation: WKNavigation?
        private var ready = false
        private var lastRequest: ReaderRequest?
        private var linkCheck: Task<Void, Never>?
        private var checkedLinks: [String: DocLinkStatus] = [:]
        private var linkURLs: [String] = []

        func cancelLinkCheck() { linkCheck?.cancel(); linkCheck = nil }

        func load(_ page: String, html: String, title: String, in webView: WKWebView) {
            cancelLinkCheck()
            checkedLinks.removeAll()
            linkURLs.removeAll()
            lastHTML = html; lastTitle = title; ready = false; lastRequest = nil
            navigation = webView.loadHTMLString(page, baseURL: nil)
        }

        func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
            guard navigation === self.navigation else { return }
            webView.callAsyncJavaScript("""
                const article = document.getElementById('devline-reader');
                if (!article) return [];
                const send = value => window.webkit.messageHandlers.readerAction.postMessage(value);
                if (typeof devlineHighlightCode === 'function') devlineHighlightCode(article, packID);
                const links = new Map();
                const originalLabels = new WeakMap();
                if (classify) {
                    window.devlineCollectLinks = () => {
                        for (const link of article.querySelectorAll('a[href]')) {
                            const raw = link.getAttribute('href');
                            if (!raw || raw.length > 8192) continue;
                            if (!links.has(raw)) links.set(raw, []);
                            links.get(raw).push(link);
                        }
                        return [...links.keys()];
                    };
                    article.addEventListener('click', event => {
                        const link = event.target.closest('a[href]');
                        if (!link) return;
                        event.preventDefault();
                        const url = link.getAttribute('href');
                        if (url && url.length <= 8192) send({action: 'link', url});
                    });
                }
                window.devlineSetLinks = statuses => {
                    for (const [url, kind, suffix] of statuses) {
                        for (const link of links.get(url) || []) {
                            if (link.dataset.linkStatus === kind) continue;
                            if (!originalLabels.has(link)) originalLabels.set(link, {
                                label: link.getAttribute('aria-label') || link.textContent.trim() || link.title,
                                title: link.title
                            });
                            const {label, title} = originalLabels.get(link);
                            link.dataset.linkStatus = kind;
                            if (label) link.setAttribute('aria-label', label + ', ' + suffix);
                            link.title = title || suffix;
                        }
                    }
                };
                window.devlineJump = (target, cue) => {
                    if (!target) return false;
                    for (let node = target.parentElement; node; node = node.parentElement) if (node.tagName === 'DETAILS') node.open = true;
                    target.scrollIntoView({block: 'start'});
                    article.querySelector('.reference-target')?.classList.remove('reference-target');
                    clearTimeout(window.devlineCueTimer);
                    if (cue) {
                        target.classList.add('reference-target');
                        window.devlineCueTimer = setTimeout(() => target.classList.remove('reference-target'), 1300);
                    }
                    return true;
                };
                window.devlineFocus = text => {
                    const normalize = value => value.replace(/\\s+/g, '').toLowerCase();
                    const needle = normalize(text);
                    if (!needle) return null;
                    return [...article.querySelectorAll('h1,h2,h3,h4,h5,h6,dt[id],pre,p,li,dd')]
                        .find(node => normalize(node.textContent).includes(needle));
                };
                for (const table of article.querySelectorAll('table')) {
                    const region = document.createElement('div');
                    region.className = 'table-scroll'; region.tabIndex = 0;
                    region.setAttribute('role', 'region'); region.setAttribute('aria-label', 'Table');
                    table.before(region); region.append(table);
                }
                for (const block of article.querySelectorAll('pre')) {
                    block.tabIndex = 0; block.setAttribute('aria-label', 'Code block');
                    const region = document.createElement('div'); region.className = 'code-region';
                    block.before(region); region.append(block);
                    const button = document.createElement('button'); button.className = 'code-copy';
                    button.type = 'button'; button.textContent = 'Copy'; button.setAttribute('aria-label', 'Copy code');
                    button.addEventListener('click', () => {
                        send({action: 'copy', text: block.textContent});
                        button.textContent = 'Copied';
                        setTimeout(() => { button.textContent = 'Copy'; }, 1400);
                    });
                    region.append(button);
                }
                const sections = [];
                const seen = new Set();
                for (const heading of article.querySelectorAll('h1,h2,h3,h4,h5,h6,dt[id]')) {
                    const title = heading.textContent.replace(/[¶#]$/g, '').trim().replace(/\\s+/g, ' ');
                    if (!title || heading.classList.contains('reader-title')) continue;
                    let id = heading.id || heading.closest('section[id]')?.id;
                    if (!id) { id = 'devline-heading-' + sections.length; heading.id = id; }
                    if (seen.has(id)) { id = 'devline-heading-' + sections.length; heading.id = id; }
                    seen.add(id);
                    sections.push({id, title, depth: heading.tagName === 'DT' ? 4 : Number(heading.tagName.slice(1))});
                }
                const positions = sections.map(section => document.getElementById(section.id)).filter(Boolean);
                // A scroll changes heading coordinates but does not change which headings are rendered.
                let visiblePositions = null;
                article.addEventListener('toggle', () => { visiblePositions = null; }, true);
                window.devlineUserScroll = false;
                for (const type of ['wheel', 'touchstart', 'pointerdown']) document.addEventListener(type, () => { window.devlineUserScroll = true; }, {passive: true});
                document.addEventListener('keydown', event => {
                    if (['ArrowUp','ArrowDown','PageUp','PageDown','Home','End',' '].includes(event.key)) window.devlineUserScroll = true;
                });
                let pending = false, semanticPosition = null, previousWidth = window.innerWidth;
                window.addEventListener('resize', () => {
                    visiblePositions = null;
                    if (window.innerWidth !== previousWidth && semanticPosition) {
                        const target = document.getElementById(semanticPosition.anchor);
                        if (target) window.scrollBy(0, target.getBoundingClientRect().top - semanticPosition.offset);
                    }
                    previousWidth = window.innerWidth;
                });
                document.addEventListener('scroll', () => {
                    if (pending) return;
                    pending = true;
                    setTimeout(() => {
                        pending = false;
                        const max = document.documentElement.scrollHeight - window.innerHeight;
                        const visible = visiblePositions ??= positions.filter(node => node.getClientRects().length);
                        let low = 0, high = visible.length - 1, active = null;
                        while (low <= high) {
                            const middle = Math.floor((low + high) / 2);
                            if (visible[middle].getBoundingClientRect().top <= 70) { active = visible[middle].id; low = middle + 1; }
                            else high = middle - 1;
                        }
                        const offset = active ? document.getElementById(active).getBoundingClientRect().top : null;
                        semanticPosition = active ? {anchor: active, offset} : null;
                        send({action: 'position', fraction: max > 0 ? window.scrollY / max : 0, anchor: active, offset, userInitiated: window.devlineUserScroll});
                    }, 120);
                }, {passive: true});
                return sections;
                """, arguments: ["packID": packID, "classify": linkStatuses != nil], in: nil, in: Self.readerWorld) { [weak self, weak webView] result in
                    guard let self, let webView, navigation === self.navigation else { return }
                    let entries = (try? result.get()) as? [[String: Any]] ?? []
                    self.reportSections?(entries.compactMap { entry in
                        guard let id = entry["id"] as? String, let title = entry["title"] as? String else { return nil }
                        return ReaderSection(title: title, target: .anchor(id), depth: entry["depth"] as? Int ?? 2)
                    })
                    self.ready = true
                    self.applyRequest(in: webView)
                    guard self.linkStatuses != nil else { return }
                    webView.callAsyncJavaScript("return window.devlineCollectLinks?.() ?? []", arguments: [:],
                                                in: nil, in: Self.readerWorld) { [weak self, weak webView] result in
                        guard let self, let webView, navigation === self.navigation,
                              let urls = (try? result.get()) as? [String] else { return }
                        self.linkURLs = urls
                        self.refreshLinks(in: webView)
                    }
                }
        }

        func refreshLinks(in webView: WKWebView) {
            cancelLinkCheck()
            checkedLinks.removeAll()
            let urls = linkURLs
            guard let linkStatuses, !urls.isEmpty else { return }
            linkCheck = Task(priority: .utility) { @MainActor [weak self, weak webView] in
                for offset in stride(from: 0, to: urls.count, by: 256) {
                    guard !Task.isCancelled else { return }
                    let batch = Array(urls[offset..<min(offset + 256, urls.count)])
                    guard let statuses = try? await linkStatuses(batch), !Task.isCancelled,
                          let self, let webView else { return }
                    for status in statuses { self.checkedLinks[status.url] = status }
                    let markers = statuses.map { [$0.url, $0.kind.rawValue, $0.kind.label] }
                    webView.callAsyncJavaScript("window.devlineSetLinks?.(statuses)", arguments: ["statuses": markers],
                                                in: nil, in: Self.readerWorld, completionHandler: nil)
                }
            }
        }

        func userContentController(_ userContentController: WKUserContentController, didReceive message: WKScriptMessage) {
            guard message.name == "readerAction", message.frameInfo.isMainFrame,
                  let value = message.body as? [String: Any], let action = value["action"] as? String else { return }
            switch action {
            case "link":
                if let raw = value["url"] as? String, raw.utf8.count <= 8192 { navigate?(raw, checkedLinks[raw]) }
            case "copy":
                guard let text = value["text"] as? String, text.utf8.count <= 2_000_000 else { return }
                copyToPasteboard(text)
            case "position":
                if let fraction = value["fraction"] as? Double, fraction.isFinite {
                    let offset = value["offset"] as? Double
                    reportPosition?(min(1, max(0, fraction)), value["anchor"] as? String, offset?.isFinite == true ? offset : nil, value["userInitiated"] as? Bool ?? false)
                }
            default: break
            }
        }

        func applyRequest(in webView: WKWebView) {
            guard ready, request != lastRequest else { return }
            lastRequest = request
            guard let request else {
                webView.evaluateJavaScript("window.getSelection()?.removeAllRanges()", in: nil, in: Self.readerWorld)
                return
            }
            webView.evaluateJavaScript("window.devlineUserScroll = false", in: nil, in: Self.readerWorld)
            switch request.action {
            case .focus(let text):
                webView.callAsyncJavaScript("""
                    return window.devlineJump(window.devlineFocus(text), cue);
                    """, arguments: ["text": text, "cue": request.cue], in: nil, in: Self.readerWorld, completionHandler: nil)
            case .restore(let position, let anchor, let offset):
                webView.callAsyncJavaScript("""
                    const target = anchor ? document.getElementById(anchor) : null;
                    if (target && offset !== null) window.scrollBy(0, target.getBoundingClientRect().top - offset);
                    else window.scrollTo(0, Math.max(0, document.documentElement.scrollHeight - window.innerHeight) * fraction);
                    """, arguments: ["fraction": min(1, max(0, position)), "anchor": anchor as Any? ?? NSNull(),
                                       "offset": offset as Any? ?? NSNull()], in: nil, in: Self.readerWorld)
            case .jump(let section):
                guard case .anchor(let id) = section.target else { return }
                webView.callAsyncJavaScript("""
                    const target = document.getElementById(anchor) || [...document.getElementsByName(anchor)][0];
                    return window.devlineJump(target || (text ? window.devlineFocus(text) : null), cue);
                    """, arguments: ["anchor": id, "cue": request.cue, "text": section.fallbackText ?? ""],
                    in: nil, in: Self.readerWorld, completionHandler: nil)
            case .find(let text, let backwards, let fromStart):
                if fromStart {
                    webView.callAsyncJavaScript("""
                        const article = document.getElementById('devline-reader');
                        const selection = window.getSelection();
                        if (!article || !selection) return;
                        const range = document.createRange(); range.selectNodeContents(article); range.collapse(true);
                        selection.removeAllRanges(); selection.addRange(range);
                        """, in: nil, in: Self.readerWorld) { [weak self, weak webView] _ in
                            guard let self, let webView, self.lastRequest == request else { return }
                            self.find(text, backwards: backwards, request: request, in: webView)
                        }
                } else { find(text, backwards: backwards, request: request, in: webView) }
            }
        }

        private func find(_ text: String, backwards: Bool, request: ReaderRequest, in webView: WKWebView) {
            let configuration = WKFindConfiguration(); configuration.wraps = true; configuration.backwards = backwards
            webView.find(text, configuration: configuration) { [weak self] result in
                guard self?.lastRequest == request else { return }
                self?.report?(request, result.matchFound)
            }
        }

        func webView(_ webView: WKWebView, decidePolicyFor action: WKNavigationAction,
                     decisionHandler: @escaping (WKNavigationActionPolicy) -> Void) {
            guard let url = action.request.url else { decisionHandler(.cancel); return }
            let address = url.absoluteString
            if address == "about:blank", action.navigationType != .linkActivated {
                decisionHandler(.allow); return
            }
            decisionHandler(.cancel)
            guard action.navigationType == .linkActivated else { return }
            if address.hasPrefix("about:blank#"), let fragment = url.fragment {
                var components = URLComponents(string: sourceURL)
                components?.fragment = fragment.removingPercentEncoding ?? fragment
                if let resolved = components?.url { navigate?(resolved.absoluteString, checkedLinks[resolved.absoluteString]) }
            } else if let safe = safeExternalURL(address) { navigate?(safe.absoluteString, checkedLinks[safe.absoluteString]) }
        }
    }
}
