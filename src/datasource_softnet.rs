use prometheus::GaugeVec;
use std::fs;
use std::sync::OnceLock;

struct SoftnetMetrics {
    softnet: GaugeVec,
}

impl SoftnetMetrics {
    fn new() -> Self {
        Self {
            softnet: prometheus::register_gauge_vec!(
                "softnet",
                "Per-CPU counters from /proc/net/softnet_stat",
                &["cpu", "field"]
            )
            .expect("register softnet"),
        }
    }
}

static SOFTNET_METRICS: OnceLock<SoftnetMetrics> = OnceLock::new();

fn metrics() -> &'static SoftnetMetrics {
    SOFTNET_METRICS.get_or_init(SoftnetMetrics::new)
}

fn parse_hex_u64(value: &str) -> Option<u64> {
    u64::from_str_radix(value, 16).ok()
}

fn parse_column(columns: &[&str], index: usize) -> Option<u64> {
    columns.get(index).and_then(|value| parse_hex_u64(value))
}

fn cpu_index_for_row(columns: &[&str], row: usize) -> u64 {
    parse_column(columns, 12).unwrap_or(row as u64)
}

pub fn update_metrics() {
    let contents = match fs::read_to_string("/proc/net/softnet_stat") {
        Ok(contents) => contents,
        Err(_) => return,
    };

    // The number of rows follows CPU hotplug.
    metrics().softnet.reset();

    for (row, line) in contents.lines().enumerate() {
        let columns: Vec<&str> = line.split_whitespace().collect();
        if columns.is_empty() {
            continue;
        }

        let cpu_id = cpu_index_for_row(&columns, row);
        let cpu_label = cpu_id.to_string();
        let metric = &metrics().softnet;
        let set_metric = |field: &str, value: u64| {
            metric
                .with_label_values(&[cpu_label.as_str(), field])
                .set(value as f64);
        };

        if let Some(value) = parse_column(&columns, 0) {
            set_metric("softnet_processed_counter", value);
        }
        if let Some(value) = parse_column(&columns, 1) {
            set_metric("softnet_dropped_counter", value);
        }
        if let Some(value) = parse_column(&columns, 2) {
            set_metric("softnet_time_squeeze_counter", value);
        }
        if let Some(value) = parse_column(&columns, 9) {
            set_metric("softnet_received_rps_counter", value);
        }
        if let Some(value) = parse_column(&columns, 10) {
            set_metric("softnet_flow_limit_count_counter", value);
        }
        if let Some(value) = parse_column(&columns, 11) {
            set_metric("softnet_backlog_len_total", value);
        }
        set_metric("softnet_cpu_index", cpu_id);
        if let Some(value) = parse_column(&columns, 13) {
            set_metric("softnet_input_qlen", value);
        }
        if let Some(value) = parse_column(&columns, 14) {
            set_metric("softnet_process_qlen", value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_index_uses_column_12_when_present() {
        // Row 1 on a system where CPU 1 is offline: column 12 reports CPU 2.
        let line = "00000010 00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000 0000000a 00000000 00000000";
        let cols: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(cpu_index_for_row(&cols, 1), 10);
    }

    #[test]
    fn cpu_index_falls_back_to_row_on_short_lines() {
        let cols = ["00000010", "00000000", "00000000"];
        assert_eq!(cpu_index_for_row(&cols, 3), 3);
    }
}
