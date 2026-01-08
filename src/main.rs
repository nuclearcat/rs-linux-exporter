#[macro_use]
extern crate rocket;

mod datasource_procfs;
mod datasource_cpufreq;
mod datasource_softnet;
mod datasource_conntrack;
mod datasource_ethtool;
mod datasource_filesystems;
mod datasource_hwmon;
mod datasource_thermal;
mod datasource_rapl;
mod datasource_power_supply;
mod datasource_nvme;
mod datasource_edac;
mod config;
mod runtime;

use prometheus::{Encoder, IntCounter, TextEncoder};
use rocket::http::ContentType;
use rocket::Config;
use std::sync::OnceLock;
use crate::config::AppConfig;

static METRICS_REQUESTS_TOTAL: OnceLock<IntCounter> = OnceLock::new();
static APP_CONFIG: OnceLock<AppConfig> = OnceLock::new();
static IS_ROOT: OnceLock<bool> = OnceLock::new();

fn metrics_requests_total() -> &'static IntCounter {
    METRICS_REQUESTS_TOTAL.get_or_init(|| {
        prometheus::register_int_counter!(
            "metrics_requests_total",
            "Total number of /metrics requests"
        )
        .expect("register metrics_requests_total")
    })
}

fn app_config() -> &'static AppConfig {
    APP_CONFIG.get_or_init(AppConfig::load)
}

fn is_root() -> bool {
    *IS_ROOT.get_or_init(|| unsafe { libc::geteuid() == 0 })
}

fn update_metrics() {
    let config = app_config();

    if config.is_datasource_enabled("procfs") {
        datasource_procfs::update_metrics(config);
    }
    if config.is_datasource_enabled("cpufreq") {
        datasource_cpufreq::update_metrics();
    }
    if config.is_datasource_enabled("softnet") {
        datasource_softnet::update_metrics();
    }
    if config.is_datasource_enabled("conntrack") {
        datasource_conntrack::update_metrics();
    }
    if config.is_datasource_enabled("filesystems") {
        datasource_filesystems::update_metrics(config);
    }
    if config.is_datasource_enabled("hwmon") {
        datasource_hwmon::update_metrics();
    }
    if config.is_datasource_enabled("thermal") {
        datasource_thermal::update_metrics();
    }
    if config.is_datasource_enabled("rapl") {
        datasource_rapl::update_metrics();
    }
    if config.is_datasource_enabled("power_supply") {
        datasource_power_supply::update_metrics();
    }
    if config.is_datasource_enabled("nvme") {
        datasource_nvme::update_metrics();
    }
    if config.is_datasource_enabled("edac") {
        datasource_edac::update_metrics();
    }
    // TODO: Implementation in progress; ethtool netlink stats disabled for now.
    // TODO: numa - /sys/devices/system/node/ (NUMA node memory stats)
}

#[get("/metrics")]
fn metrics() -> (ContentType, String) {
    metrics_requests_total().inc();
    update_metrics();

    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder
        .encode(&metric_families, &mut buffer)
        .expect("encode metrics");

    (ContentType::Plain, String::from_utf8(buffer).unwrap_or_default())
}

#[get("/")]
fn index() -> &'static str {
    "rs-linux-exporter: /metrics"
}

#[launch]
fn rocket() -> _ {
    runtime::init();
    if runtime::debug_enabled() {
        eprintln!("Debug logging enabled.");
    }
    if !is_root() {
        eprintln!("\x1b[31mNon-root: ethtool stats collection disabled.\x1b[0m");
    }
    let figment = Config::figment().merge(("port", 9100));
    rocket::custom(figment).mount("/", routes![index, metrics])
}

#[cfg(test)]
mod tests {
    use super::rocket;
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    #[test]
    fn index_returns_hint() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/").dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_string().unwrap_or_default(),
            "rs-linux-exporter: /metrics"
        );
    }

    #[test]
    fn metrics_endpoint_returns_ok() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/metrics").dispatch();

        assert_eq!(response.status(), Status::Ok);
    }

    #[test]
    fn metrics_endpoint_exposes_prometheus_text() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/metrics").dispatch();

        let body = response.into_string().unwrap_or_default();
        assert!(body.contains("metrics_requests_total"));
    }

    #[test]
    fn metrics_endpoint_contains_help_and_type() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/metrics").dispatch();

        let body = response.into_string().unwrap_or_default();
        // Prometheus format requires HELP and TYPE lines
        assert!(body.contains("# HELP"));
        assert!(body.contains("# TYPE"));
    }

    #[test]
    fn metrics_endpoint_increments_counter() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");

        // First request
        let response1 = client.get("/metrics").dispatch();
        let body1 = response1.into_string().unwrap_or_default();

        // Find metrics_requests_total value
        let count1 = extract_counter_value(&body1, "metrics_requests_total");

        // Second request
        let response2 = client.get("/metrics").dispatch();
        let body2 = response2.into_string().unwrap_or_default();

        let count2 = extract_counter_value(&body2, "metrics_requests_total");

        // Counter should have incremented
        assert!(count2 > count1, "Counter should increment: {} -> {}", count1, count2);
    }

    #[test]
    fn metrics_endpoint_has_correct_content_type() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/metrics").dispatch();

        let content_type = response.content_type();
        assert!(content_type.is_some());
        assert_eq!(content_type.unwrap().to_string(), "text/plain; charset=utf-8");
    }

    #[test]
    fn unknown_endpoint_returns_404() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/unknown").dispatch();

        assert_eq!(response.status(), Status::NotFound);
    }

    /// Helper to extract counter value from Prometheus text format
    fn extract_counter_value(body: &str, metric_name: &str) -> u64 {
        for line in body.lines() {
            if line.starts_with(metric_name) && !line.starts_with('#') {
                // Format: metric_name value
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    return parts[1].parse().unwrap_or(0);
                }
            }
        }
        0
    }
}
