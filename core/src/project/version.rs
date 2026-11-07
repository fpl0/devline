use crate::updates::doc_release;

#[derive(Clone, Copy, Debug)]
pub(super) enum Grammar {
    Cargo,
    Constraint,
}

#[derive(Clone, Debug)]
pub(super) enum Rule {
    Any,
    Exact(String),
    Series(Vec<String>, usize),
    Constraint(String, Grammar),
    Unknown,
}

#[derive(Clone, Copy, Debug)]
struct Number {
    parts: [u32; 4],
    len: usize,
}

impl Number {
    fn parse(value: &str) -> Option<Self> {
        let value = value.trim().trim_start_matches('v');
        let mut parts = [0; 4];
        let mut len = 0;
        for component in value.split('.') {
            if component.is_empty() || !component.bytes().all(|byte| byte.is_ascii_digit()) {
                return None;
            }
            *parts.get_mut(len)? = component.parse().ok()?;
            len += 1;
        }
        Some(Self { parts, len })
    }

    fn before_increment(self, candidate: Self, component: usize) -> bool {
        let mut upper = self.parts;
        let Some(incremented) = upper[component].checked_add(1) else {
            return false;
        };
        upper[component] = incremented;
        upper[component + 1..].fill(0);
        candidate.parts < upper
    }
}

pub(super) fn numeric(value: &str) -> bool {
    Number::parse(value).is_some()
}

pub(super) fn matches(rule: &Rule, version: &str) -> Option<bool> {
    let (release, _) = doc_release(version);
    match rule {
        Rule::Any | Rule::Unknown => None,
        Rule::Exact(value) => Some(
            value.trim_start_matches('v') == release
                || Number::parse(value)
                    .zip(Number::parse(release))
                    .is_some_and(|(expected, actual)| expected.parts == actual.parts),
        ),
        Rule::Series(values, width) => {
            let candidate = Number::parse(release)?;
            let mut compatible = true;
            for value in values {
                let expected = Number::parse(value)?;
                let width = (*width).min(expected.len);
                compatible &=
                    candidate.len >= width && expected.parts[..width] == candidate.parts[..width];
            }
            Some(compatible)
        }
        Rule::Constraint(expression, grammar) => {
            let candidate = Number::parse(release)?;
            constraint(expression, *grammar, candidate)
        }
    }
}

fn constraint(expression: &str, grammar: Grammar, candidate: Number) -> Option<bool> {
    let expression = expression.trim();
    if expression == "*" {
        return Some(true);
    }
    if expression.starts_with(['[', '(']) && expression.ends_with([']', ')']) {
        let lower_inclusive = expression.starts_with('[');
        let upper_inclusive = expression.ends_with(']');
        let range = &expression[1..expression.len() - 1];
        if let Some((start, end)) = range.split_once(',') {
            let lower = if start.trim().is_empty() {
                true
            } else {
                let start = Number::parse(start)?.parts;
                candidate.parts > start || (lower_inclusive && candidate.parts == start)
            };
            let upper = if end.trim().is_empty() {
                true
            } else {
                let end = Number::parse(end)?.parts;
                candidate.parts < end || (upper_inclusive && candidate.parts == end)
            };
            return Some(lower && upper);
        }
        return (lower_inclusive && upper_inclusive)
            .then(|| Number::parse(range).map(|exact| candidate.parts == exact.parts))
            .flatten();
    }

    // The supported comparator forms are intentionally bounded. An expression
    // we cannot interpret never silently selects a newer release.
    let mut any_match = false;
    for alternatives in expression.split("||") {
        let mut all_match = true;
        for part in alternatives.split(',') {
            let part = part.trim();
            let (operator, value) = [">=", "<=", "==", "!=", "~>", "~=", ">", "<", "=", "^", "~"]
                .into_iter()
                .find_map(|operator| {
                    part.strip_prefix(operator)
                        .map(|value| (operator, value.trim()))
                })
                .unwrap_or(("", part));
            if let Some(prefix) = value.strip_suffix(".*") {
                let prefix = Number::parse(prefix)?;
                let equal = candidate.parts[..prefix.len] == prefix.parts[..prefix.len];
                all_match &= match operator {
                    "" | "=" | "==" => equal,
                    "!=" => !equal,
                    _ => return None,
                };
                continue;
            }
            let boundary = Number::parse(value)?;
            let actual = candidate.parts;
            let bound = boundary.parts;
            let caret = || {
                let component = bound[..boundary.len]
                    .iter()
                    .position(|value| *value != 0)
                    .unwrap_or(boundary.len - 1);
                actual >= bound && boundary.before_increment(candidate, component)
            };
            all_match &= match operator {
                ">=" => actual >= bound,
                "<=" => actual <= bound,
                ">" => actual > bound,
                "<" => actual < bound,
                "!=" => actual != bound,
                "~>" | "~=" => {
                    actual >= bound
                        && boundary.before_increment(candidate, boundary.len.saturating_sub(2))
                }
                "^" => caret(),
                "~" => {
                    actual >= bound
                        && boundary.before_increment(candidate, (boundary.len - 1).min(1))
                }
                "" if matches!(grammar, Grammar::Cargo) => caret(),
                _ => actual == bound,
            };
        }
        any_match |= all_match;
    }
    Some(any_match)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_compare_the_upstream_version_and_leave_unknowns_explicit() {
        assert_eq!(
            matches(&Rule::Exact("0.162.0".into()), "0.162.0+devline.4"),
            Some(true)
        );
        assert_eq!(
            matches(&Rule::Exact("0.162.0".into()), "0.162.1+devline.1"),
            Some(false)
        );
        assert_eq!(
            matches(&Rule::Series(vec!["26".into()], 1), "26.0.2.1"),
            Some(true)
        );
        assert_eq!(
            matches(&Rule::Series(vec!["8.0".into(), "10.0".into()], 2), "10.0"),
            Some(false)
        );
        assert_eq!(
            matches(
                &Rule::Constraint(">=3.12,<3.14".into(), Grammar::Constraint),
                "3.15.2"
            ),
            Some(false)
        );
        assert_eq!(
            matches(
                &Rule::Constraint("~> 8.1.3".into(), Grammar::Constraint),
                "8.1.6"
            ),
            Some(true)
        );
        assert_eq!(
            matches(&Rule::Constraint("0.8.9".into(), Grammar::Cargo), "0.9.1"),
            Some(false)
        );
        assert_eq!(
            matches(
                &Rule::Constraint("[4.1,4.2)".into(), Grammar::Constraint),
                "4.1.2"
            ),
            Some(true)
        );
        assert_eq!(
            matches(
                &Rule::Constraint("git+https://example.invalid".into(), Grammar::Constraint),
                "3.15.2"
            ),
            None
        );
    }
}
