use clap::Parser;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::PathBuf;

const MAX_LINE: u64 = 1024 * 1024;

#[derive(Parser)]
#[command(
    version,
    about = "Count scalar JSON values at a field across a JSON Lines stream"
)]
struct Args {
    /// JSON Pointer, for example /event/name.
    #[arg(long, value_parser = pointer)]
    field: String,
    /// File to read, or - for standard input.
    #[arg(default_value = "-", value_hint = clap::ValueHint::FilePath)]
    input: PathBuf,
}

fn pointer(value: &str) -> Result<String, String> {
    if value.is_empty()
        || value.starts_with('/')
            && value
                .split('~')
                .skip(1)
                .all(|suffix| suffix.starts_with(['0', '1']))
    {
        Ok(value.into())
    } else {
        Err("Use a JSON Pointer such as /event/name (escape ~ as ~0 and / as ~1)".into())
    }
}

#[derive(Serialize)]
struct Report {
    records: u64,
    missing_or_compound: u64,
    counts: Vec<Count>,
}

#[derive(Serialize)]
struct Count {
    value: Value,
    count: u64,
}

fn summarize(mut input: impl BufRead, field: &str) -> Result<Report, Box<dyn std::error::Error>> {
    let mut line = Vec::new();
    let mut line_number = 0;
    let mut records = 0;
    let mut missing_or_compound = 0;
    let mut counts = BTreeMap::<String, Count>::new();
    loop {
        line.clear();
        let read = input
            .by_ref()
            .take(MAX_LINE + 1)
            .read_until(b'\n', &mut line)?;
        if read == 0 {
            break;
        }
        line_number += 1;
        if read as u64 > MAX_LINE {
            return Err(format!("line {line_number} exceeds 1 MiB").into());
        }
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        let item: Value = serde_json::from_slice(&line)
            .map_err(|error| format!("line {line_number}: {error}"))?;
        records += 1;
        if let Some(value) = item
            .pointer(field)
            .filter(|value| !value.is_array() && !value.is_object())
        {
            let key = serde_json::to_string(value)?;
            counts
                .entry(key)
                .and_modify(|entry| entry.count += 1)
                .or_insert_with(|| Count {
                    value: value.clone(),
                    count: 1,
                });
        } else {
            missing_or_compound += 1;
        }
    }
    let mut counts: Vec<_> = counts.into_values().collect();
    counts.sort_by_key(|entry| std::cmp::Reverse(entry.count));
    Ok(Report {
        records,
        missing_or_compound,
        counts,
    })
}

fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let input: Box<dyn BufRead> = if args.input.as_os_str() == "-" {
        Box::new(io::stdin().lock())
    } else {
        Box::new(BufReader::new(File::open(&args.input)?))
    };
    let report = summarize(input, &args.field)?;
    let mut output = io::stdout().lock();
    serde_json::to_writer(&mut output, &report)?;
    writeln!(output)?;
    Ok(())
}

fn main() {
    if let Err(error) = run(Args::parse()) {
        eprintln!("jsonl-report: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_stable_order_and_keeps_scalar_types_distinct() {
        let input =
            b"{\"status\":\"200\"}\n{\"status\":200}\n{}\n{\"status\":200}\n{\"status\":[]}\n\n";
        let result = summarize(&input[..], "/status").unwrap();
        assert_eq!(result.records, 5);
        assert_eq!(result.missing_or_compound, 2);
        assert_eq!(result.counts[0].value, 200);
        assert_eq!(result.counts[0].count, 2);
        assert_eq!(result.counts[1].value, "200");
    }

    #[test]
    fn malformed_and_oversize_records_identify_the_line() {
        assert!(
            summarize(&b"{}\n!\n"[..], "")
                .err()
                .unwrap()
                .to_string()
                .contains("line 2")
        );
        let huge = vec![b'x'; MAX_LINE as usize + 1];
        assert!(
            summarize(&huge[..], "")
                .err()
                .unwrap()
                .to_string()
                .contains("line 1 exceeds")
        );
    }
}
