use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::{BufRead, Read},
};

const MAX_LINE: u64 = 1_048_576;
const BOUNDS: [f64; 16] = [
    1., 5., 10., 25., 50., 100., 250., 500., 1000., 2500., 5000., 10000., 30000., 60000., 300000.,
    3600000.,
];

#[derive(Deserialize)]
struct Request {
    method: String,
    route: String,
    status: u16,
    duration_ms: f64,
    #[serde(default)]
    response_bytes: u64,
}
impl Request {
    fn validate(&self) -> Result<(), String> {
        if !["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"]
            .contains(&self.method.as_str())
        {
            return Err("method must be a standard uppercase HTTP method".into());
        }
        if !self.route.starts_with('/')
            || self.route.len() > 512
            || self.route.chars().any(char::is_control)
        {
            return Err(
                "route must start with / and contain at most 512 bytes without control characters"
                    .into(),
            );
        }
        if !(100..=599).contains(&self.status)
            || !self.duration_ms.is_finite()
            || !(0.0..=3600000.).contains(&self.duration_ms)
        {
            return Err("invalid HTTP status or duration_ms (expected 0–3600000)".into());
        }
        Ok(())
    }
}

#[derive(Default)]
struct Aggregate {
    requests: u64,
    errors: u64,
    client_errors: u64,
    bytes: u64,
    total: f64,
    max: f64,
    buckets: [u64; 16],
}
impl Aggregate {
    fn add(&mut self, r: &Request) -> Result<(), String> {
        self.bytes = self
            .bytes
            .checked_add(r.response_bytes)
            .ok_or("response byte count overflow")?;
        self.requests += 1;
        self.errors += u64::from(r.status >= 500);
        self.client_errors += u64::from((400..500).contains(&r.status));
        self.total += r.duration_ms;
        self.max = self.max.max(r.duration_ms);
        let index = BOUNDS
            .iter()
            .position(|bound| r.duration_ms <= *bound)
            .expect("validated duration");
        self.buckets[index] += 1;
        Ok(())
    }
    fn percentile(&self, percent: u64) -> Option<f64> {
        if self.requests == 0 {
            return None;
        }
        let rank = (self.requests * percent).div_ceil(100);
        let mut count = 0;
        for (index, bucket) in self.buckets.iter().enumerate() {
            count += bucket;
            if count >= rank {
                return Some(BOUNDS[index]);
            }
        }
        None
    }
    fn finish(self) -> Metrics {
        Metrics {
            requests: self.requests,
            server_errors: self.errors,
            client_errors: self.client_errors,
            response_bytes: self.bytes,
            error_rate_percent: if self.requests == 0 {
                0.
            } else {
                100. * self.errors as f64 / self.requests as f64
            },
            mean_ms: if self.requests == 0 {
                None
            } else {
                Some(self.total / self.requests as f64)
            },
            max_ms: if self.requests == 0 {
                None
            } else {
                Some(self.max)
            },
            p50_upper_ms: self.percentile(50),
            p95_upper_ms: self.percentile(95),
        }
    }
}
#[derive(Debug, Serialize)]
pub struct Metrics {
    pub requests: u64,
    pub server_errors: u64,
    pub client_errors: u64,
    pub response_bytes: u64,
    pub error_rate_percent: f64,
    pub mean_ms: Option<f64>,
    pub max_ms: Option<f64>,
    pub p50_upper_ms: Option<f64>,
    pub p95_upper_ms: Option<f64>,
}
#[derive(Debug, Serialize)]
pub struct Route {
    pub method: String,
    pub route: String,
    #[serde(flatten)]
    pub metrics: Metrics,
}
#[derive(Debug, Serialize)]
pub struct Report {
    pub schema_version: u32,
    pub summary: Metrics,
    pub skipped_records: u64,
    pub warnings: Vec<String>,
    pub routes: Vec<Route>,
}

/// Process one record at a time. Memory is bounded by line size and route count,
/// not by the number of requests; percentiles are histogram upper bounds.
pub fn analyze(
    mut input: impl BufRead,
    skip_invalid: bool,
    max_routes: usize,
) -> Result<Report, String> {
    if !(1..=10000).contains(&max_routes) {
        return Err("max_routes must be 1–10000".into());
    }
    let mut line = Vec::new();
    let mut number = 0;
    let mut skipped = 0;
    let mut warnings = Vec::new();
    let mut total = Aggregate::default();
    let mut routes = BTreeMap::<(String, String), Aggregate>::new();
    loop {
        line.clear();
        let size = input
            .by_ref()
            .take(MAX_LINE + 1)
            .read_until(b'\n', &mut line)
            .map_err(|e| e.to_string())?;
        if size == 0 {
            break;
        }
        number += 1;
        if size as u64 > MAX_LINE {
            return Err(format!("line {number} exceeds 1 MiB"));
        }
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        let parsed = serde_json::from_slice::<Request>(&line)
            .map_err(|e| e.to_string())
            .and_then(|r| r.validate().map(|()| r));
        let request = match parsed {
            Ok(r) => r,
            Err(e) => {
                let message = format!("line {number}: {e}");
                if !skip_invalid {
                    return Err(message);
                }
                skipped += 1;
                if warnings.len() < 5 {
                    warnings.push(message);
                }
                continue;
            }
        };
        let key = (request.method.clone(), request.route.clone());
        if !routes.contains_key(&key) && routes.len() >= max_routes {
            return Err(format!(
                "more than {max_routes} routes; log route templates or increase --max-routes"
            ));
        }
        routes.entry(key).or_default().add(&request)?;
        total.add(&request)?;
    }
    let mut rows: Vec<_> = routes
        .into_iter()
        .map(|((method, route), value)| Route {
            method,
            route,
            metrics: value.finish(),
        })
        .collect();
    rows.sort_by(|a, b| {
        b.metrics
            .requests
            .cmp(&a.metrics.requests)
            .then_with(|| a.method.cmp(&b.method))
            .then_with(|| a.route.cmp(&b.route))
    });
    Ok(Report {
        schema_version: 1,
        summary: total.finish(),
        skipped_records: skipped,
        warnings,
        routes: rows,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aggregates_routes_statuses_and_histogram_bounds() {
        let input = br#"{"method":"GET","route":"/orders/{id}","status":200,"duration_ms":2,"response_bytes":20}
{"method":"GET","route":"/orders/{id}","status":503,"duration_ms":80,"response_bytes":10}
{"method":"POST","route":"/orders","status":400,"duration_ms":8}
"#;
        let r = analyze(&input[..], false, 100).unwrap();
        assert_eq!(r.summary.requests, 3);
        assert_eq!(r.summary.server_errors, 1);
        assert_eq!(r.summary.client_errors, 1);
        assert_eq!(r.summary.response_bytes, 30);
        assert_eq!(r.summary.mean_ms, Some(30.));
        assert_eq!(r.summary.p50_upper_ms, Some(10.));
        assert_eq!(r.summary.p95_upper_ms, Some(100.));
        assert_eq!(r.routes[0].route, "/orders/{id}");
    }
    #[test]
    fn malformed_records_are_explicit_and_bounded() {
        assert!(
            analyze(&b"{}\n"[..], false, 10)
                .unwrap_err()
                .contains("line 1")
        );
        let r = analyze(&b"{}\nnot json\n\n"[..], true, 10).unwrap();
        assert_eq!(r.skipped_records, 2);
        assert_eq!(r.summary.requests, 0);
        assert_eq!(r.summary.p95_upper_ms, None);
        let huge = vec![b'x'; MAX_LINE as usize + 1];
        assert!(analyze(&huge[..], true, 10).is_err());
    }
    #[test]
    fn route_limit_and_invalid_duration_fail() {
        let input = br#"{"method":"GET","route":"/a","status":200,"duration_ms":1}
{"method":"GET","route":"/b","status":200,"duration_ms":1}"#;
        assert!(
            analyze(&input[..], false, 1)
                .unwrap_err()
                .contains("more than 1 routes")
        );
        assert!(
            analyze(
                &br#"{"method":"GET","route":"/","status":200,"duration_ms":-1}"#[..],
                false,
                10
            )
            .is_err()
        );
    }
}
