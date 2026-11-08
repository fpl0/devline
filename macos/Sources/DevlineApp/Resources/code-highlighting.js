const devlineDocsetLanguages = new Map(Object.entries({
    aspnet: 'csharp', axum: 'rust', chi: 'go', csharp: 'csharp', ecto: 'elixir',
    elixir: 'elixir', fastapi: 'python', go: 'go', java: 'java', node: 'javascript',
    phoenix: 'elixir', playwright: 'typescript', python: 'python', rails: 'ruby',
    react: 'javascript', ruby: 'ruby', rust: 'rust', swift: 'swift', swiftui: 'swift',
    typescript: 'typescript', vite: 'typescript'
}));

const devlinePlainLanguages = new Set(['none', 'nohighlight', 'no-highlight', 'plain', 'plaintext', 'text', 'txt', 'output', 'mermaid']);
const devlineLanguageAliases = new Map([['dotnetcli', 'bash'], ['iex', 'elixir']]);

function devlineCodeLanguage(block) {
    for (const element of [block.querySelector(':scope > code'), block, block.parentElement]) {
        if (!element) continue;
        const classes = Array.from(element.classList, name => name.toLowerCase());
        if (classes.some(name => devlinePlainLanguages.has(name))) return 'plaintext';
        const attribute = element.getAttribute('data-lang') || element.getAttribute('data-language');
        if (attribute?.trim()) return attribute.trim().toLowerCase();
        const languageClass = classes.find(name => /^(?:language|lang)-.+/.test(name));
        if (languageClass) return languageClass.replace(/^(?:language|lang)-/, '');
        if (element === block.parentElement) continue;
        const makeup = classes.indexOf('makeup');
        if (makeup !== -1 && classes[makeup + 1]) return classes[makeup + 1];
        const known = classes.find(name => hljs.getLanguage(name));
        if (known) return known;
    }
    return null;
}

function devlineTextNodes(root) {
    const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
    const nodes = [];
    while (walker.nextNode()) nodes.push(walker.currentNode);
    return nodes;
}

function devlineApplyCodeTokens(block, source, markup) {
    const template = document.createElement('template');
    // Highlight.js escapes the source. Parse its output inertly and only copy token classes.
    template.innerHTML = markup;
    if (template.content.textContent !== source) return;

    const tokens = [];
    let offset = 0;
    for (const node of devlineTextNodes(template.content)) {
        const end = offset + node.length;
        const style = node.parentElement?.closest('span[class*="hljs-"]')?.className;
        if (style && end > offset) tokens.push({start: offset, end, style});
        offset = end;
    }
    if (!tokens.length) return;

    // Project tokens onto the original text nodes so code links and anchors survive.
    let tokenIndex = 0;
    offset = 0;
    for (const node of devlineTextNodes(block)) {
        const start = offset;
        offset += node.length;
        if (node.length === 0 || !tokens[tokenIndex] || tokens[tokenIndex].start >= offset) continue;

        const fragment = document.createDocumentFragment();
        let cursor = start;
        while (tokens[tokenIndex]?.start < offset) {
            const token = tokens[tokenIndex];
            const from = Math.max(token.start, start);
            const to = Math.min(token.end, offset);
            if (cursor < from) fragment.append(node.data.slice(cursor - start, from - start));
            const span = document.createElement('span');
            span.className = token.style;
            span.textContent = node.data.slice(from - start, to - start);
            fragment.append(span);
            cursor = to;
            if (token.end > offset) break;
            tokenIndex++;
        }
        if (cursor < offset) fragment.append(node.data.slice(cursor - start));
        node.replaceWith(fragment);
    }
    block.setAttribute('data-devline-highlighted', '');
}

function devlineHighlightCode(article, packID) {
    const fallback = devlineDocsetLanguages.get(packID.replace(/^devdocs-/, ''));
    for (const block of article.querySelectorAll('pre')) {
        if (block.hasAttribute('data-devline-highlighted')) continue;
        const explicit = devlineCodeLanguage(block);
        let language = explicit ?? fallback;
        if (!language || devlinePlainLanguages.has(language)) continue;
        const source = block.textContent;
        if (!source.trim() || source.length > 32_768) continue;
        if (!explicit && fallback === 'python' && /^\s*>>> /m.test(source)) language = 'python-repl';
        language = devlineLanguageAliases.get(language) ?? language;
        if (!hljs.getLanguage(language)) continue;

        const result = hljs.highlight(source, {language, ignoreIllegals: Boolean(explicit)});
        if (!result.illegal && (explicit || result.relevance > 0)) {
            devlineApplyCodeTokens(block, source, result.value);
        }
    }
}
