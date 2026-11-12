use clap::{Parser, ValueEnum};
use jsonl_report::analyze;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
    path::PathBuf,
};
#[derive(Clone, ValueEnum)]
enum Format {
    Text,
    Json,
}
#[derive(Parser)]
#[command(version, about = "Logbook: bounded-memory HTTP request-log analysis")]
struct Args {
    /// JSON Lines request log, or - for standard input.
    #[arg(default_value = "-")]
    input: PathBuf,
    #[arg(long, value_enum, default_value = "text")]
    format: Format,
    /// Skip malformed records, count them, and include the first five errors.
    #[arg(long)]
    skip_invalid: bool,
    /// Maximum distinct method/route pairs.
    #[arg(long, default_value = "1000")]
    max_routes: usize,
    /// Exit 2 if the observed HTTP 5xx percentage exceeds this threshold.
    #[arg(long)]
    fail_error_rate: Option<f64>,
}
fn run(args: Args) -> Result<i32, Box<dyn std::error::Error>> {
    if args
        .fail_error_rate
        .is_some_and(|n| !n.is_finite() || !(0.0..=100.0).contains(&n))
    {
        return Err("--fail-error-rate must be 0–100".into());
    }
    let input: Box<dyn BufRead> = if args.input.as_os_str() == "-" {
        Box::new(io::stdin().lock())
    } else {
        Box::new(BufReader::new(File::open(args.input)?))
    };
    let report = analyze(input, args.skip_invalid, args.max_routes)?;
    let mut out = io::stdout().lock();
    match args.format {
        Format::Json => {
            serde_json::to_writer_pretty(&mut out, &report)?;
            writeln!(out)?;
        }
        Format::Text => {
            writeln!(
                out,
                "LOGBOOK · REQUEST HEALTH\n{} requests · {} server errors ({:.2}%) · {} client errors · {} skipped",
                report.summary.requests,
                report.summary.server_errors,
                report.summary.error_rate_percent,
                report.summary.client_errors,
                report.skipped_records
            )?;
            writeln!(
                out,
                "\n{:7} {:7} {:10} {:10} ROUTE",
                "COUNT", "5XX", "MEAN MS", "P95 ≤ MS"
            )?;
            for row in &report.routes {
                writeln!(
                    out,
                    "{:7} {:7} {:10.2} {:10.0} {} {}",
                    row.metrics.requests,
                    row.metrics.server_errors,
                    row.metrics.mean_ms.unwrap_or(0.),
                    row.metrics.p95_upper_ms.unwrap_or(0.),
                    row.method,
                    row.route
                )?;
            }
            for warning in &report.warnings {
                writeln!(out, "Warning: {warning}")?;
            }
            writeln!(
                out,
                "\nPercentiles are histogram upper bounds, not exact measurements."
            )?;
        }
    }
    Ok(
        if args
            .fail_error_rate
            .is_some_and(|n| report.summary.error_rate_percent > n)
        {
            2
        } else {
            0
        },
    )
}
fn main() {
    match run(Args::parse()) {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("logbook: {error}");
            std::process::exit(1)
        }
    }
}
