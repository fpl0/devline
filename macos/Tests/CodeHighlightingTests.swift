import AppKit
import WebKit

@main @MainActor
struct CodeHighlightingTests {
    static func main() {
        do {
            try run()
        } catch {
            FileHandle.standardError.write(Data("Code highlighting test failed: \(error)\n".utf8))
            exit(1)
        }
    }

    private static func run() throws {
        _ = NSApplication.shared
        let resources = URL(fileURLWithPath: "macos/Sources/DevKitApp/Resources", isDirectory: true)
        let script = try ["highlight.min.js", "code-highlighting.js"].map { name in
            try String(contentsOf: resources.appendingPathComponent(name), encoding: .utf8)
        }.joined(separator: "\n")

        let world = WKContentWorld.world(name: "DevlineArticle")
        let configuration = WKWebViewConfiguration()
        configuration.defaultWebpagePreferences.allowsContentJavaScript = false
        configuration.websiteDataStore = .nonPersistent()
        configuration.userContentController.addUserScript(WKUserScript(
            source: script, injectionTime: .atDocumentEnd, forMainFrameOnly: true, in: world
        ))
        let view = WKWebView(frame: .zero, configuration: configuration)
        let test = TestNavigation(world: world)
        view.navigationDelegate = test
        view.loadHTMLString(page, baseURL: nil)

        let deadline = Date(timeIntervalSinceNow: 15)
        while test.result == nil && Date() < deadline {
            RunLoop.current.run(until: min(deadline, Date(timeIntervalSinceNow: 0.05)))
        }
        guard let result = test.result else { throw Failure(description: "Timed out loading the reader fixture") }
        guard let failures = try result.get() as? [String] else {
            throw Failure(description: "WebKit returned an unexpected test result")
        }
        guard failures.isEmpty else { throw Failure(description: failures.joined(separator: "\n")) }
        print("Code highlighting: language tags, legacy examples, preserved text and links, and isolated script execution passed")
    }

    private struct Failure: Error, CustomStringConvertible {
        let description: String
    }

    private final class TestNavigation: NSObject, WKNavigationDelegate {
        let world: WKContentWorld
        var result: Result<Any, Error>?

        init(world: WKContentWorld) { self.world = world }

        func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
            webView.callAsyncJavaScript(checks, in: nil, in: world) { self.result = $0 }
        }

        func webView(_ webView: WKWebView, didFail navigation: WKNavigation!, withError error: Error) {
            result = .failure(error)
        }

        func webView(_ webView: WKWebView, didFailProvisionalNavigation navigation: WKNavigation!, withError error: Error) {
            result = .failure(error)
        }
    }

    private static let page = #"""
    <!doctype html><html lang="en"><meta charset="utf-8">
    <meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src data:; style-src 'unsafe-inline'; base-uri 'none'; form-action 'none'">
    <body><main id="devline-reader">
    <script>document.body.dataset.pageScriptRan = 'true';</script>
    <pre id="typed"><code class="language-js">const value = "&lt;/script&gt;&lt;img src=x onerror=alert(1)&gt;";
    const greeting = "Olá 👋";</code></pre>
    <pre id="legacy">d<span id="split-token">e</span>f <a id="reference" href="#definition">render</a>(value):
        return value &lt; 10
    </pre>
    <pre id="attribute" data-lang="json"><code>{"count": 10}</code></pre>
    <pre id="plain"><code class="language-text">def literal(): return 10</code></pre>
    <pre id="unknown"><code class="language-unrecognized">def literal(): return 10</code></pre>
    <p id="definition">The <code id="inline">return value</code> expression.</p>
    </main></body></html>
    """#

    private static let checks = #"""
    const article = document.getElementById('devline-reader');
    const blocks = [...article.querySelectorAll('pre')];
    const before = blocks.map(block => block.textContent);
    const link = document.getElementById('reference');
    devlineHighlightCode(article, 'fastapi');

    const failures = [];
    const check = (condition, message) => { if (!condition) failures.push(message); };
    const get = id => document.getElementById(id);
    check(get('typed').hasAttribute('data-devline-highlighted'), 'The explicitly tagged block was not highlighted');
    check(get('typed').querySelector('.hljs-keyword')?.textContent === 'const', 'The JavaScript tag did not override the pack default');
    check(get('typed').querySelector('.hljs-string')?.textContent === '"</script><img src=x onerror=alert(1)>"',
          'Escaped markup did not remain a literal string token');
    check(!article.querySelector('img'), 'The highlighter interpreted code as markup');
    check(get('legacy').hasAttribute('data-devline-highlighted'), 'The legacy Python block was not highlighted');
    check([...get('legacy').querySelectorAll('.hljs-keyword')].slice(0, 3).map(node => node.textContent).join('') === 'def' &&
          get('split-token').querySelector('.hljs-keyword')?.textContent === 'e',
          'The Python keyword did not keep its color across existing markup');
    check(get('reference') === link && link.getAttribute('href') === '#definition', 'An existing reference link changed');
    check(get('attribute').querySelector('.hljs-attr')?.textContent === '"count"', 'The data-lang attribute was ignored');
    for (const [index, block] of blocks.entries()) {
        check(get(block.id) === block && block.textContent === before[index], 'Code text or anchor changed for ' + block.id);
    }
    for (const id of ['plain', 'unknown', 'inline']) {
        check(!get(id).hasAttribute('data-devline-highlighted') && !get(id).querySelector('span'),
              'An unsupported or inline example was highlighted: ' + id);
    }
    check(!document.body.dataset.pageScriptRan, 'An article script ran inside the reader');
    return failures;
    """#
}
