use anyhow::{Context, Result};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use serde_json::Value;
use std::{
    env,
    io::{self, IsTerminal, Write},
    path::Path,
    process::{Command, Stdio},
};

pub fn safe_text(text: &str) -> String {
    text.chars()
        .filter(|ch| !ch.is_control() || matches!(ch, '\n' | '\t'))
        .collect()
}

pub(super) fn shell_quote(input: &str) -> String {
    format!("'{}'", input.replace('\'', "'\\''"))
}

fn value<'a>(article: &'a Value, key: &str) -> &'a str {
    article[key].as_str().unwrap_or_default()
}

fn heading_key(text: &str) -> String {
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

#[derive(Default)]
struct Markdown {
    output: String,
    lists: Vec<Option<u64>>,
    quote_depth: usize,
    code: bool,
    table_cell: bool,
    first_heading: bool,
}

impl Markdown {
    fn write(&mut self, text: &str) {
        for line in text.split_inclusive('\n') {
            if (self.output.is_empty() || self.output.ends_with('\n')) && line != "\n" {
                self.output.push_str(&"│ ".repeat(self.quote_depth));
            }
            self.output.push_str(line);
        }
    }

    fn newline(&mut self) {
        if !self.output.is_empty() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }
    }

    fn paragraph(&mut self) {
        self.newline();
        if !self.output.is_empty() && !self.output.ends_with("\n\n") {
            self.output.push('\n');
        }
    }

    fn render(body: &str, title: &str) -> String {
        let mut result = Self::default();
        let options = Options::ENABLE_TABLES
            | Options::ENABLE_TASKLISTS
            | Options::ENABLE_FOOTNOTES
            | Options::ENABLE_HEADING_ATTRIBUTES
            | Options::ENABLE_GFM;
        for event in Parser::new_ext(body, options) {
            match event {
                Event::Start(Tag::Heading { .. }) => {
                    result.paragraph();
                    result.first_heading = result.output.is_empty();
                }
                Event::End(TagEnd::Heading(_)) => {
                    if result.first_heading && result.output.trim().eq_ignore_ascii_case(title) {
                        result.output.clear();
                    } else {
                        result.paragraph();
                    }
                }
                Event::End(TagEnd::Paragraph) if result.lists.is_empty() => result.paragraph(),
                Event::End(TagEnd::Paragraph | TagEnd::Item) => result.newline(),
                Event::Start(Tag::List(start)) => {
                    result.newline();
                    result.lists.push(start);
                }
                Event::End(TagEnd::List(_)) => {
                    result.lists.pop();
                    if result.lists.is_empty() {
                        result.paragraph();
                    }
                }
                Event::Start(Tag::Item) => {
                    result.newline();
                    let mut prefix = "  ".repeat(result.lists.len().saturating_sub(1));
                    if let Some(Some(number)) = result.lists.last_mut() {
                        prefix.push_str(&format!("{number}. "));
                        *number = number.saturating_add(1);
                    } else {
                        prefix.push_str("• ");
                    }
                    result.write(&prefix);
                }
                Event::Start(Tag::CodeBlock(_)) => {
                    result.paragraph();
                    result.code = true;
                }
                Event::End(TagEnd::CodeBlock) => {
                    result.code = false;
                    result.paragraph();
                }
                Event::Text(text) if result.code => {
                    let padding = "  ".repeat(result.lists.len() + 1);
                    for line in text.lines() {
                        result.write(&padding);
                        result.write(line);
                        result.output.push('\n');
                    }
                }
                Event::Text(text) | Event::Code(text) => result.write(&text),
                Event::Start(Tag::BlockQuote(kind)) => {
                    result.paragraph();
                    result.quote_depth += 1;
                    if let Some(kind) = kind {
                        result.write(&format!("{kind:?}\n"));
                    }
                }
                Event::End(TagEnd::BlockQuote(_)) => {
                    result.quote_depth -= 1;
                    result.paragraph();
                }
                Event::Start(Tag::TableHead | Tag::TableRow) => {
                    result.newline();
                    result.table_cell = false;
                }
                Event::Start(Tag::TableCell) => {
                    if result.table_cell {
                        result.write("  │  ");
                    }
                    result.table_cell = true;
                }
                Event::End(TagEnd::TableHead | TagEnd::TableRow) => result.newline(),
                Event::End(TagEnd::Table) => result.paragraph(),
                Event::TaskListMarker(true) => result.write("[x] "),
                Event::TaskListMarker(false) => result.write("[ ] "),
                Event::SoftBreak => result.write(" "),
                Event::HardBreak => result.newline(),
                Event::Rule => {
                    result.paragraph();
                    result.write("────────────────");
                    result.paragraph();
                }
                _ => {}
            }
        }
        safe_text(&result.output)
    }
}

fn article_text(article: &Value, formatted: bool) -> (String, Option<usize>) {
    let title = value(article, "title");
    let pack = value(article, "pack_id");
    let version = value(article, "version");
    let source = value(article, "source_url");
    let body = value(article, "body");
    let custom_label = article["terminal_label"].as_str();
    if !formatted {
        let label = custom_label.map_or_else(|| format!("{pack} · {version}"), str::to_owned);
        return (format!("{title}\n{label}\n{source}\n\n{body}"), None);
    }
    let mut label = custom_label.map_or_else(|| format!("{pack}@{version}"), str::to_owned);
    if let (Some(part), Some(parts)) = (article["part"].as_u64(), article["parts"].as_u64()) {
        if parts > 1 {
            label.push_str(&format!(" · Part {part} of {parts}"));
        }
    }
    let header = safe_text(&format!("{title}\n{label}\n{source}\n\n"));
    let body = if let Some(markdown) = article["terminal_markdown"].as_str() {
        Markdown::render(markdown, title)
    } else {
        safe_text(body)
    };
    let focus_line = article["focus_text"]
        .as_str()
        .map(safe_text)
        .filter(|focus| !focus.trim().is_empty())
        .and_then(|focus| {
            let focus = focus.trim();
            body.lines()
                .position(|line| line.trim() == focus)
                .or_else(|| body.lines().position(|line| line.contains(focus)))
        })
        .or_else(|| {
            let anchor = heading_key(article["anchor"].as_str()?);
            if anchor.is_empty() {
                return None;
            }
            body.lines().position(|line| heading_key(line) == anchor)
        })
        .map(|index| header.lines().count() + index + 1);
    let mut article_text = header + &body;
    for (key, title) in [("previous_id", "Previous part"), ("next_id", "Next part")] {
        if let Some(id) = article[key].as_str() {
            let id = shell_quote(&safe_text(id));
            article_text.push_str(&format!("\n\n{title}\n  devline docs show {id}"));
        }
    }
    (article_text, focus_line)
}

fn send_to_pager(stdin: &mut impl Write, text: &str) -> io::Result<()> {
    // main enables normal UNIX SIGPIPE for shell pipelines. Early pager exit is
    // different: keep the interactive command alive even if its child closes stdin.
    #[cfg(unix)]
    let previous = unsafe { libc::signal(libc::SIGPIPE, libc::SIG_IGN) };
    let result = stdin.write_all(text.as_bytes());
    #[cfg(unix)]
    if previous != libc::SIG_ERR {
        unsafe { libc::signal(libc::SIGPIPE, previous) };
    }
    match result {
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        other => other,
    }
}

pub fn interactive(no_pager: bool) -> bool {
    !no_pager
        && io::stdout().is_terminal()
        && env::var("TERM").is_ok_and(|term| !term.is_empty() && term != "dumb")
}

pub fn read(article: &Value, no_pager: bool) -> Result<()> {
    let formatted = interactive(no_pager);
    let (text, focus_line) = article_text(article, formatted);
    if formatted {
        let configured = env::var("DEVLINE_PAGER").or_else(|_| env::var("PAGER"));
        let preference = configured.as_deref().unwrap_or("/usr/bin/less");
        let mut args = preference.split_ascii_whitespace();
        if let Some(pager) = args.next().filter(|pager| *pager != "cat") {
            let mut command = Command::new(pager);
            if Path::new(pager)
                .file_name()
                .is_some_and(|name| name == "less")
            {
                if configured.is_err() {
                    command.arg("-F");
                }
                if let Some(line) = focus_line {
                    command.arg(format!("+{line}"));
                }
            }
            command.args(args);
            if let Ok(mut child) = command.stdin(Stdio::piped()).spawn() {
                if let Some(mut stdin) = child.stdin.take() {
                    send_to_pager(&mut stdin, &text).context("Could not write to pager")?;
                }
                if child.wait()?.success() {
                    return Ok(());
                }
            }
        }
    }
    println!("{text}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn terminal_markdown_is_readable_and_focus_uses_display_lines() {
        let mut article = json!({
            "title":"First Steps", "pack_id":"fastapi", "version":"1", "source_url":"https://example.com",
            "focus_text":"def root():",
            "body":"# First Steps\n\nOpen [the guide](https://example.com).\n\n```python\ndef root():\n    return 1\n```\n\n- Test \u{1b}[2J\n",
        });
        article["terminal_markdown"] = article["body"].clone();
        let (rendered, focus) = article_text(&article, true);
        assert_eq!(rendered.matches("First Steps").count(), 1);
        assert_eq!(
            rendered.lines().nth(focus.unwrap() - 1).unwrap(),
            "  def root():"
        );
        assert!(rendered.contains("Open the guide."));
        assert!(rendered.contains("• Test"));
        assert!(!rendered.contains('\u{1b}'));
        assert!(!rendered.contains("```"));
        assert!(article_text(&article, false)
            .0
            .ends_with(value(&article, "body")));
    }
}
