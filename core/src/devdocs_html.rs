//! Convert untrusted DevDocs HTML into an offline reader fragment and searchable text.

use anyhow::{bail, Result};
use base64::Engine as _;
use fluent_uri::{Iri, IriRef};
use html5ever::{
    tendril::StrTendril,
    tokenizer::{
        states::RawKind, BufferQueue, TagKind, Token, TokenSink, TokenSinkResult, Tokenizer,
        TokenizerOpts,
    },
};
use percent_encoding::percent_decode_str;
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashSet},
    fmt::Write as _,
};

const MAX_DEPTH: usize = 128;
pub(super) const MAX_IMAGE_BYTES: u64 = 750_000;
const VOID: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];
const OMIT: &[&str] = &[
    "audio", "button", "canvas", "embed", "footer", "form", "head", "iframe", "input", "nav",
    "noscript", "object", "script", "style", "svg", "template", "video",
];
const PRESENTATION: &[&str] = &[
    "headerlink",
    "visually-hidden",
    "sr-only",
    "sidebar",
    "copy-button",
    "copy-path",
    "breadcrumb",
    "breadcrumbs",
    "pagination",
    "screen-reader-only",
    "toc",
    "source-link",
    "search-toggle",
    "view-source",
    "icon-action",
    "skip-link",
    "hover-link",
];
const BLOCK: &[&str] = &[
    "article",
    "blockquote",
    "dd",
    "div",
    "dl",
    "dt",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "hr",
    "li",
    "main",
    "ol",
    "p",
    "pre",
    "section",
    "summary",
    "table",
    "tr",
    "ul",
];
const SAFE_TAGS: &[&str] = &[
    "a",
    "abbr",
    "article",
    "aside",
    "b",
    "blockquote",
    "br",
    "caption",
    "code",
    "col",
    "colgroup",
    "dd",
    "del",
    "details",
    "dfn",
    "div",
    "dl",
    "dt",
    "em",
    "figcaption",
    "figure",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "hr",
    "i",
    "img",
    "kbd",
    "li",
    "mark",
    "ol",
    "p",
    "pre",
    "s",
    "samp",
    "section",
    "small",
    "span",
    "strong",
    "sub",
    "summary",
    "sup",
    "table",
    "tbody",
    "td",
    "th",
    "thead",
    "tfoot",
    "tr",
    "u",
    "ul",
    "var",
    "wbr",
];
const SAFE_CLASSES: &[&str] = &[
    "admonition",
    "admonition-title",
    "attention",
    "caution",
    "danger",
    "error",
    "example",
    "hint",
    "important",
    "info",
    "note",
    "tip",
    "warning",
    "versionadded",
    "versionchanged",
    "deprecated",
    "versionmodified",
    "sourceCode",
    "sig",
    "signature",
    "_attribution",
    "_attribution-link",
];

#[derive(Default)]
struct Element {
    tag: String,
    attributes: BTreeMap<String, String>,
    children: Vec<Node>,
}

enum Node {
    Text(String),
    Element(Element),
}

impl Element {
    fn attr(&self, name: &str) -> &str {
        self.attributes.get(name).map(String::as_str).unwrap_or("")
    }

    fn classes(&self) -> impl Iterator<Item = &str> {
        self.attr("class").split_whitespace()
    }

    fn descendants(&self) -> impl Iterator<Item = &Element> {
        fn collect<'a>(node: &'a Element, elements: &mut Vec<&'a Element>) {
            elements.push(node);
            for child in &node.children {
                if let Node::Element(element) = child {
                    collect(element, elements);
                }
            }
        }
        let mut elements = Vec::new();
        collect(self, &mut elements);
        elements.into_iter()
    }
}

struct Parser {
    stack: Vec<Element>,
    too_deep: bool,
}

impl Parser {
    fn pop(&mut self) {
        let node = self.stack.pop().expect("open HTML element");
        if let Some(parent) = self.stack.last_mut() {
            parent.children.push(Node::Element(node));
        }
    }
}

struct Sink(RefCell<Parser>);

impl TokenSink for Sink {
    type Handle = ();

    fn process_token(&self, token: Token, _: u64) -> TokenSinkResult<()> {
        let mut parser = self.0.borrow_mut();
        if parser.too_deep {
            return TokenSinkResult::Continue;
        }
        match token {
            Token::TagToken(tag) if tag.kind == TagKind::StartTag => {
                let name = tag.name.to_string();
                let is_void = VOID.contains(&name.as_str()) || tag.self_closing;
                let raw = match name.as_str() {
                    "script" => Some(RawKind::ScriptData),
                    "style" | "iframe" | "noscript" | "xmp" => Some(RawKind::Rawtext),
                    "title" | "textarea" => Some(RawKind::Rcdata),
                    _ => None,
                };
                if parser.stack.len() == MAX_DEPTH {
                    parser.too_deep = true;
                    return TokenSinkResult::Continue;
                }
                parser.stack.push(Element {
                    tag: name,
                    attributes: tag
                        .attrs
                        .into_iter()
                        .map(|attr| (attr.name.local.to_string(), attr.value.to_string()))
                        .collect(),
                    children: Vec::new(),
                });
                if is_void {
                    parser.pop();
                } else if let Some(raw) = raw {
                    return TokenSinkResult::RawData(raw);
                }
            }
            Token::TagToken(tag) => {
                if let Some(index) = parser
                    .stack
                    .iter()
                    .rposition(|node| node.tag == tag.name.as_ref())
                {
                    while parser.stack.len() > index {
                        parser.pop();
                    }
                }
            }
            Token::CharacterTokens(text) => {
                let children = &mut parser.stack.last_mut().expect("HTML root").children;
                if let Some(Node::Text(existing)) = children.last_mut() {
                    existing.push_str(&text);
                } else {
                    children.push(Node::Text(text.to_string()));
                }
            }
            _ => {}
        }
        TokenSinkResult::Continue
    }
}

fn parse(raw: &str) -> Result<Element> {
    let input = BufferQueue::default();
    input.push_back(StrTendril::from_slice(raw));
    let tokenizer = Tokenizer::new(
        Sink(RefCell::new(Parser {
            stack: vec![Element::default()],
            too_deep: false,
        })),
        TokenizerOpts::default(),
    );
    let _ = tokenizer.feed(&input);
    tokenizer.end();
    let mut parser = tokenizer.sink.0.into_inner();
    if parser.too_deep {
        bail!("DevDocs article has excessively nested HTML");
    }
    while parser.stack.len() > 1 {
        parser.pop();
    }
    Ok(parser.stack.pop().expect("HTML root"))
}

fn safe_web_url(raw: &str) -> Option<Iri<&str>> {
    if raw.is_empty() || raw.chars().any(|c| c <= ' ' || c == '\\') {
        return None;
    }
    let url = Iri::parse(raw).ok()?;
    let authority = url.authority()?;
    (matches!(url.scheme().as_str(), "http" | "https")
        && authority.userinfo().is_none()
        && !authority.host().is_empty())
    .then_some(url)
}

pub(super) fn web_source(raw: &str) -> Option<String> {
    Some(safe_web_url(raw)?.strip_fragment().as_str().to_owned())
}

pub(super) fn publisher_url(raw: &str) -> Option<String> {
    let start = raw
        .rfind("class=\"_attribution\"")
        .and_then(|index| raw[..index].rfind('<'))
        .or_else(|| {
            raw.rfind("_attribution-link")
                .and_then(|index| raw[..index].rfind("<a"))
        })?;
    let tail = parse(&raw[start..]).ok()?;
    let result = tail
        .descendants()
        .find(|node| node.tag == "a" && node.classes().any(|name| name == "_attribution-link"))
        .and_then(|node| web_source(node.attr("href")));
    result
}

fn resolve_link(raw: &str, source: &str) -> Option<String> {
    if raw.is_empty() || raw.starts_with("//") || raw.chars().any(|c| c <= ' ' || c == '\\') {
        return None;
    }
    if raw.starts_with('#') {
        return Some(raw.into());
    }
    let link = IriRef::parse(raw).ok()?;
    let source = Iri::parse(source).ok()?;
    let absolute = link.resolve_against(&source).ok()?;
    safe_web_url(absolute.as_str())?;
    Some(absolute.as_str().to_owned())
}

fn language_name(value: &str) -> bool {
    value.len() <= 35
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_alphabetic())
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'+' | b'#' | b'.' | b'-')
        })
}

fn clean_anchor(value: &str) -> Option<String> {
    let anchor = percent_decode_str(value).decode_utf8().ok()?;
    (!anchor.is_empty()
        && anchor.len() <= 500
        && !anchor.chars().any(|c| c <= ' ' || c.is_control()))
    .then(|| anchor.into_owned())
}

fn sanitize(
    element: Element,
    source: &str,
    local_link: &impl Fn(&str) -> Option<String>,
    asset: &impl Fn(&str) -> Option<Vec<u8>>,
) -> Option<Element> {
    let tag = element.tag.as_str();
    if OMIT.contains(&tag) || element.classes().any(|name| PRESENTATION.contains(&name)) {
        return None;
    }

    if tag == "img" {
        let src = element.attr("src");
        let alt = element.attr("alt");
        return Some(
            if let Some(url) = asset(src).and_then(|bytes| image_url(&bytes)) {
                Element {
                    tag: "img".into(),
                    attributes: [("src".into(), url), ("alt".into(), alt.into())].into(),
                    children: Vec::new(),
                }
            } else {
                let mut image = Element {
                    tag: "span".into(),
                    children: vec![Node::Text(
                        if alt.is_empty() {
                            "Upstream image"
                        } else {
                            alt
                        }
                        .into(),
                    )],
                    ..Element::default()
                };
                if let Some(href) = resolve_link(src, source) {
                    image.tag = "a".into();
                    image.attributes.insert("href".into(), href);
                }
                image
            },
        );
    }

    let mut attributes = BTreeMap::new();
    let anchor = clean_anchor(element.attr("id")).or_else(|| {
        (tag == "a")
            .then(|| clean_anchor(element.attr("name")))
            .flatten()
    });
    if let Some(anchor) = anchor {
        attributes.insert("id".into(), anchor);
    }
    let classes = element
        .classes()
        .filter(|name| {
            SAFE_CLASSES.contains(name) || (matches!(tag, "code" | "pre") && language_name(name))
        })
        .collect::<Vec<_>>()
        .join(" ");
    if !classes.is_empty() {
        attributes.insert("class".into(), classes);
    }
    if matches!(tag, "code" | "pre") {
        for key in ["data-lang", "data-language"] {
            let value = element.attr(key);
            if language_name(value) {
                attributes.insert(key.into(), value.into());
            }
        }
    }
    if tag == "a" {
        let href = element.attr("href");
        if !element.classes().any(|name| name == "toc-backref") {
            if let Some(href) = local_link(href).or_else(|| resolve_link(href, source)) {
                attributes.insert("href".into(), href);
            }
        }
    }
    let numeric_attributes: &[&str] = match tag {
        "td" | "th" => &["colspan", "rowspan"],
        "ol" => &["start"],
        _ => &[],
    };
    for &key in numeric_attributes {
        let value = element.attr(key);
        if (1..=4).contains(&value.len()) && value.bytes().all(|byte| byte.is_ascii_digit()) {
            attributes.insert(key.into(), value.into());
        }
    }
    if tag == "details" && element.attributes.contains_key("open") {
        attributes.insert("open".into(), String::new());
    }
    let children = element
        .children
        .into_iter()
        .filter_map(|node| match node {
            Node::Text(text) => Some(Node::Text(text)),
            Node::Element(element) => {
                sanitize(element, source, local_link, asset).map(Node::Element)
            }
        })
        .collect();
    let tag = if SAFE_TAGS.contains(&tag) && !(tag == "span" && attributes.is_empty()) {
        element.tag
    } else if attributes.contains_key("id") {
        "span".into()
    } else {
        String::new()
    };
    Some(Element {
        tag,
        attributes,
        children,
    })
}

fn image_url(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() || bytes.len() as u64 > MAX_IMAGE_BYTES {
        return None;
    }
    let kind = if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        "png"
    } else if bytes.starts_with(b"\xff\xd8\xff") {
        "jpeg"
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        "gif"
    } else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
        "webp"
    } else if safe_svg(bytes) {
        "svg+xml"
    } else {
        return None;
    };
    Some(format!(
        "data:image/{kind};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}

fn safe_svg(bytes: &[u8]) -> bool {
    let Ok(raw) = std::str::from_utf8(bytes) else {
        return false;
    };
    let Ok(svg) = roxmltree::Document::parse(raw) else {
        return false;
    };
    if svg.root_element().tag_name().name() != "svg" || svg.descendants().any(|node| node.is_pi()) {
        return false;
    }
    svg.descendants()
        .filter(|node| node.is_element())
        .all(|node| {
            !matches!(
                node.tag_name().name().to_ascii_lowercase().as_str(),
                "script"
                    | "foreignobject"
                    | "image"
                    | "iframe"
                    | "object"
                    | "embed"
                    | "a"
                    | "animate"
                    | "set"
                    | "animatetransform"
                    | "animatemotion"
                    | "style"
            ) && node.attributes().all(|attribute| {
                let name = attribute.name().to_ascii_lowercase();
                let value = attribute.value().to_ascii_lowercase();
                !name.starts_with("on")
                    && !matches!(name.as_str(), "src" | "base" | "style")
                    && (name != "href" || value.starts_with('#'))
                    && !value.contains("url")
                    && !value.contains(['@', '\\'])
            })
        })
}

fn escape(value: &str, attributes: bool, output: &mut String) {
    for character in value.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' if attributes => output.push_str("&quot;"),
            '\'' if attributes => output.push_str("&#x27;"),
            _ => output.push(character),
        }
    }
}

fn render(element: &Element, output: &mut String) {
    if !element.tag.is_empty() {
        let _ = write!(output, "<{}", element.tag);
        for (name, value) in &element.attributes {
            let _ = write!(output, " {name}=\"");
            escape(value, true, output);
            output.push('"');
        }
        output.push('>');
    }
    for child in &element.children {
        match child {
            Node::Text(text) => escape(text, false, output),
            Node::Element(element) => render(element, output),
        }
    }
    if !element.tag.is_empty() && !VOID.contains(&element.tag.as_str()) {
        let _ = write!(output, "</{}>", element.tag);
    }
}

fn article_text(element: &Element, pre: bool, output: &mut String) {
    let tag = element.tag.as_str();
    if OMIT.contains(&tag) || element.classes().any(|name| PRESENTATION.contains(&name)) {
        return;
    }
    if tag == "br" {
        output.push('\n');
        return;
    }
    if tag == "img" {
        output.push_str(element.attr("alt"));
        return;
    }
    if BLOCK.contains(&tag) {
        output.push_str(if tag == "li" { "\n• " } else { "\n\n" });
    }
    if tag == "pre" {
        let mut code = String::new();
        children_text(element, true, &mut code);
        for line in code.trim_end_matches('\n').lines() {
            output.push_str("    ");
            output.push_str(line);
            output.push('\n');
        }
    } else {
        children_text(element, pre, output);
    }
    if BLOCK.contains(&tag) {
        output.push_str("\n\n");
    } else if matches!(tag, "td" | "th") {
        output.push_str("  ");
    }
}

fn children_text(element: &Element, pre: bool, output: &mut String) {
    for child in &element.children {
        match child {
            Node::Element(element) => article_text(element, pre, output),
            Node::Text(text) if pre => output.push_str(text),
            Node::Text(text) => {
                let mut space = false;
                for character in text.chars() {
                    if character.is_whitespace() {
                        if !space {
                            output.push(' ');
                        }
                        space = true;
                    } else {
                        output.push(character);
                        space = false;
                    }
                }
            }
        }
    }
}

fn clean_text(raw: &str) -> String {
    let mut result = String::new();
    for line in raw.trim().lines() {
        let line = line.trim_end();
        if !line.is_empty() || !result.ends_with("\n\n") {
            result.push_str(line);
            result.push('\n');
        }
    }
    result.trim_end().into()
}

pub(super) fn attribution(raw: &str) -> Result<String> {
    let mut text = String::new();
    article_text(&parse(raw)?, false, &mut text);
    Ok(clean_text(&text))
}

pub(super) struct Article {
    pub(super) title: String,
    pub(super) text: String,
    pub(super) html: String,
    pub(super) anchors: HashSet<String>,
}

pub(super) fn article(
    raw: &str,
    source: &str,
    local_link: impl Fn(&str) -> Option<String>,
    asset: impl Fn(&str) -> Option<Vec<u8>>,
) -> Result<Article> {
    let tree = parse(raw)?;
    let mut title = String::new();
    if let Some(heading) = tree
        .descendants()
        .find(|node| matches!(node.tag.as_str(), "h1" | "title"))
    {
        article_text(heading, false, &mut title);
    }
    let sanitized = sanitize(tree, source, &local_link, &asset).unwrap_or_default();
    let anchors = sanitized
        .descendants()
        .filter_map(|node| node.attributes.get("id").cloned())
        .collect();
    let mut text = String::new();
    article_text(&sanitized, false, &mut text);
    let mut html = String::new();
    render(&sanitized, &mut html);
    Ok(Article {
        title: clean_text(&title),
        text: clean_text(&text),
        html: html.trim().into(),
        anchors,
    })
}
