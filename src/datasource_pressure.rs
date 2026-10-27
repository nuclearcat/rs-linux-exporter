use prometheus::GaugeVec;
use std::fs;
use std::sync::OnceLock;

/// PSI resources exposed under /proc/pressure. `irq` only exists on kernels
/// built with CONFIG_IRQ_TIME_ACCOUNTING (6.1+) and only reports `full`.
const RESOURCES: &[&str] = &["cpu", "memory", "io", "irq"];

struct PressureMetrics {
    avg_percent: GaugeVec,
    stall_seconds_total: GaugeVec,
}

impl PressureMetrics {
    fn new() -> Self {
        Self {
            avg_percent: prometheus::register_gauge_vec!(
                "pressure_stall_avg_percent",
                "Share of wall time tasks were stalled on the resource, averaged over the window, from /proc/pressure",
                &["resource", "kind", "window"]
            )
            .expect("register pressure_stall_avg_percent"),
            stall_seconds_total: prometheus::register_gauge_vec!(
                "pressure_stall_seconds_total",
                "Cumulative time tasks were stalled on the resource, from /proc/pressure",
                &["resource", "kind"]
            )
            .expect("register pressure_stall_seconds_total"),
        }
    }
}

static PRESSURE_METRICS: OnceLock<PressureMetrics> = OnceLock::new();

fn metrics() -> &'static PressureMetrics {
    PRESSURE_METRICS.get_or_init(PressureMetrics::new)
}

#[derive(Debug, PartialEq)]
struct PressureLine<'a> {
    kind: &'a str,
    avg10: f64,
    avg60: f64,
    avg300: f64,
    total_us: u64,
}

/// Parses `some avg10=0.00 avg60=0.00 avg300=0.00 total=296675`.
fn parse_line(line: &str) -> Option<PressureLine<'_>> {
    let mut parts = line.split_whitespace();
    let kind = parts.next()?;
    if kind != "some" && kind != "full" {
        return None;
    }

    let (mut avg10, mut avg60, mut avg300, mut total_us) = (None, None, None, None);
    for part in parts {
        let (key, value) = part.split_once('=')?;
        match key {
            "avg10" => avg10 = value.parse().ok(),
            "avg60" => avg60 = value.parse().ok(),
            "avg300" => avg300 = value.parse().ok(),
            "total" => total_us = value.parse().ok(),
            _ => {}
        }
    }

    Some(PressureLine {
        kind,
        avg10: avg10?,
        avg60: avg60?,
        avg300: avg300?,
        total_us: total_us?,
    })
}

pub fn update_metrics() {
    let m = metrics();
    m.avg_percent.reset();
    m.stall_seconds_total.reset();

    for resource in RESOURCES {
        // Reads fail with EOPNOTSUPP when PSI is compiled in but disabled.
        let contents = match fs::read_to_string(format!("/proc/pressure/{resource}")) {
            Ok(contents) => contents,
            Err(_) => continue,
        };

        for line in contents.lines().filter_map(parse_line) {
            for (window, value) in [("10", line.avg10), ("60", line.avg60), ("300", line.avg300)] {
                m.avg_percent
                    .with_label_values(&[resource, line.kind, window])
                    .set(value);
            }
            m.stall_seconds_total
                .with_label_values(&[resource, line.kind])
                .set(line.total_us as f64 / 1_000_000.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_some_and_full_lines() {
        assert_eq!(
            parse_line("some avg10=1.25 avg60=0.50 avg300=0.10 total=296675"),
            Some(PressureLine {
                kind: "some",
                avg10: 1.25,
                avg60: 0.50,
                avg300: 0.10,
                total_us: 296675,
            })
        );
        assert_eq!(
            parse_line("full avg10=0.00 avg60=0.00 avg300=0.00 total=0").map(|l| l.kind),
            Some("full")
        );
    }

    #[test]
    fn rejects_unknown_kind_and_incomplete_lines() {
        assert_eq!(
            parse_line("half avg10=0.00 avg60=0.00 avg300=0.00 total=1"),
            None
        );
        assert_eq!(parse_line("some avg10=0.00 avg60=0.00 total=1"), None);
        assert_eq!(parse_line(""), None);
    }
}
