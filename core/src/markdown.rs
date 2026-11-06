use fluent_uri::{Iri, IriRef};
use pulldown_cmark::{html, CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use std::{borrow::Cow, collections::HashSet};

pub fn matches(pack: &str, path: &str, rules: Option<&[String]>) -> bool {
    if let Some(rules) = rules {
        return rules.iter().any(|rule| {
            rule.split_once('*')
                .map_or(path == rule, |(prefix, suffix)| {
                    path.strip_prefix(prefix)
                        .is_some_and(|tail| tail.ends_with(suffix))
                })
        });
    }
    // Older catalogs predate the format selector. These packs came from Markdown sources.
    match pack {
        "fastapi" | "csharp" | "aspnet" | "rails" => true,
        "chi" => path == "README.md",
        "ruby" => path.ends_with(".md") || path.starts_with("api/") || path.starts_with("method/"),
        _ => false,
    }
}

fn options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_HEADING_ATTRIBUTES
        | Options::ENABLE_GFM
}

pub(super) fn web_url(url: &Iri<&str>) -> bool {
    let scheme = url.scheme().as_str();
    (scheme.eq_ignore_ascii_case("https") || scheme.eq_ignore_ascii_case("http"))
        && url
            .authority()
            .is_some_and(|authority| !authority.host().is_empty() && authority.userinfo().is_none())
}

fn destination(raw: &str, source: Option<&Iri<&str>>) -> Option<String> {
    if raw.is_empty() || raw.starts_with("//") {
        return None;
    }
    let reference = IriRef::parse(raw).ok()?;
    if raw.starts_with('#') {
        return Some(raw.to_owned());
    }
    if reference.has_scheme() {
        return web_url(&Iri::parse(raw).ok()?).then(|| raw.to_owned());
    }
    let resolved = reference.resolve_against(&source?.strip_fragment()).ok()?;
    web_url(&resolved.borrow()).then(|| resolved.into_string())
}

fn inline_text<'a>(events: impl IntoIterator<Item = &'a Event<'a>>) -> String {
    let mut text = String::new();
    for event in events {
        match event {
            Event::Text(text_part) | Event::Code(text_part) => text.push_str(text_part),
            Event::SoftBreak | Event::HardBreak => text.push(' '),
            _ => {}
        }
    }
    text
}

fn slug(text: &str) -> String {
    text.trim()
        .chars()
        .flat_map(char::to_lowercase)
        .filter_map(|ch| {
            if ch.is_alphanumeric() || matches!(ch, '-' | '_') {
                Some(ch)
            } else {
                ch.is_whitespace().then_some('-')
            }
        })
        .collect()
}

pub fn render(body: &str, source_url: &str, focus: Option<&str>) -> (String, Option<String>) {
    let source = Iri::parse(source_url).ok().filter(web_url);
    let mut links = Vec::new();
    let mut in_code_block = false;
    let mut focus_is_code = false;
    let mut events: Vec<_> = Parser::new_ext(body, options())
        .filter_map(|event| match event {
            Event::Html(_)
            | Event::InlineHtml(_)
            | Event::Start(Tag::HtmlBlock | Tag::Image { .. })
            | Event::End(TagEnd::HtmlBlock | TagEnd::Image) => None,
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            }) => {
                let url = destination(&dest_url, source.as_ref());
                links.push(url.is_some());
                url.map(|url| {
                    Event::Start(Tag::Link {
                        link_type,
                        dest_url: url.into(),
                        title,
                        id,
                    })
                })
            }
            Event::End(TagEnd::Link) => links.pop().unwrap_or(false).then_some(event),
            _ => {
                match &event {
                    Event::Start(Tag::CodeBlock(_)) => in_code_block = true,
                    Event::End(TagEnd::CodeBlock) => in_code_block = false,
                    Event::Text(text) if in_code_block => {
                        focus_is_code |= focus.is_some_and(|focus| text.contains(focus));
                    }
                    _ => {}
                }
                Some(event)
            }
        })
        .collect();

    let mut heading_ids = HashSet::new();
    for index in 0..events.len() {
        let Event::Start(Tag::Heading { level, id, .. }) = &events[index] else {
            continue;
        };
        let generated = || {
            slug(&inline_text(events[index + 1..].iter().take_while(
                |event| !matches!(event, Event::End(TagEnd::Heading(_))),
            )))
        };
        let mut base = id.as_ref().map_or_else(generated, |id| id.to_string());
        if base.is_empty() {
            base = "section".into();
        }
        let mut unique = base.clone();
        let mut suffix = 1;
        while !heading_ids.insert(unique.clone()) {
            unique = format!("{base}-{suffix}");
            suffix += 1;
        }
        // The parser escapes ids. Other author-supplied attributes never reach the HTML.
        events[index] = Event::Start(Tag::Heading {
            level: *level,
            id: Some(unique.into()),
            classes: Vec::new(),
            attrs: Vec::new(),
        });
    }

    let focus = focus.map(|focus| {
        if focus_is_code {
            return focus.to_owned();
        }
        let events: Vec<_> = Parser::new_ext(focus, options()).collect();
        let visible = inline_text(&events);
        if visible.trim().is_empty() {
            focus.to_owned()
        } else {
            visible.trim().to_owned()
        }
    });
    let mut output = String::with_capacity(body.len());
    html::push_html(&mut output, events.into_iter());
    (output, focus)
}

// A stored document can have been split at a blank line inside a fenced example.
// Return the opening fence only if the last block reached the end without a closer.
pub fn continuation_fence(preceding: &str) -> Option<&str> {
    let span = Parser::new_ext(preceding, options())
        .into_offset_iter()
        .filter_map(|(event, span)| {
            matches!(
                event,
                Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(_)))
            )
            .then_some(span)
        })
        .last()?;
    if span.end < preceding.trim_end().len() {
        return None;
    }
    let mut lines = preceding[span].trim_end().lines();
    let opening = lines.next()?.trim_start();
    let marker = opening.as_bytes().first().copied()?;
    let width = opening.bytes().take_while(|byte| *byte == marker).count();
    if !matches!(marker, b'`' | b'~') || width < 3 {
        return None;
    }
    let closed = lines.last().is_some_and(|line| {
        let line = line.trim();
        line.len() >= width && line.bytes().all(|byte| byte == marker)
    });
    (!closed).then_some(opening)
}

pub fn with_continuation<'a>(body: &'a str, preceding: &str) -> Cow<'a, str> {
    continuation_fence(preceding)
        .map(|opening| Cow::Owned(format!("{opening}\n{body}")))
        .unwrap_or(Cow::Borrowed(body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_gfm_and_keeps_only_safe_reader_links() {
        let source = r#"# **Quick** start { #setup onclick=alert .untrusted }

## Use `HTTP`

## Use `HTTP`

> [!TIP]
> Good to know.

| Mode | Value |
| --- | --- |
| ~~old~~ | **new** |

- [x] Prepared
- `literal <script>`

[jump](#setup) [guide](../start/?lang=en#cli) [vendor](https://example.org/start/)
[script](javascript:alert(1)) [encoded](jav&#x61;script:alert(1)) [disk](file:///etc/passwd)
[network](//example.net/) [bad authority](https://user:pass@example.net/)
![Logo](https://example.org/logo.svg) <img src="https://example.org/tracker" onerror="alert(1)">

<script>document.location = 'https://example.org/track';</script>
"#;
        let (html, focus) = render(
            source,
            "https://example.org/guides/setup/#original",
            Some("# **Quick** start { #setup onclick=alert .untrusted }"),
        );
        for expected in [
            "<h1 id=\"setup\"><strong>Quick</strong> start</h1>",
            "id=\"use-http\"",
            "id=\"use-http-1\"",
            "<table>",
            "<del>old</del>",
            "type=\"checkbox\"",
            "href=\"#setup\"",
            "href=\"https://example.org/guides/start/?lang=en#cli\"",
            "href=\"https://example.org/start/\"",
            "literal &lt;script&gt;",
            "Logo",
        ] {
            assert!(html.contains(expected), "missing {expected}: {html}");
        }
        for forbidden in [
            "onclick",
            "untrusted",
            "<script",
            "<img",
            "href=\"javascript",
            "file://",
            "user:pass",
            "example.net",
            "tracker",
            "logo.svg",
        ] {
            assert!(!html.contains(forbidden), "found {forbidden}: {html}");
        }
        assert_eq!(focus.as_deref(), Some("Quick start"));
    }

    #[test]
    fn format_selectors_are_explicit_and_can_override_legacy_defaults() {
        assert!(matches("ruby", "method/Array#map", None));
        assert!(!matches("ruby", "doc/syntax/methods.rdoc", None));
        assert!(!matches("chi", "api/middleware", None));
        assert!(!matches("go", "std/net/http", None));
        assert!(!matches("fastapi", "tutorial.md", Some(&[])));
        let rules = ["guides/*.md".to_owned()];
        assert!(matches("example", "guides/intro.md", Some(&rules)));
        assert!(!matches("example", "guides/intro.txt", Some(&rules)));
    }
}
