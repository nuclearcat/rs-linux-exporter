#[macro_use]
extern crate rocket;

mod datasource_procfs;
mod datasource_cpufreq;
mod datasource_softnet;
mod datasource_conntrack;
mod datasource_ethtool;
mod datasource_filesystems;
mod datasource_hwmon;
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
    datasource_procfs::update_metrics(app_config());
    datasource_cpufreq::update_metrics();
    datasource_softnet::update_metrics();
    datasource_conntrack::update_metrics();
    datasource_filesystems::update_metrics(app_config());
    datasource_hwmon::update_metrics();
    // TODO: Implementation in progress; ethtool netlink stats disabled for now.
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
        assert_eq!(response.into_string().unwrap_or_default(), "rs-linux-exporter: /metrics");
    }

    #[test]
    fn metrics_endpoint_exposes_prometheus_text() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/metrics").dispatch();

        assert_eq!(response.status(), Status::Ok);
        let body = response.into_string().unwrap_or_default();
        assert!(body.contains("metrics_requests_total"));
    }
}
