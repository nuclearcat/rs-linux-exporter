#[macro_use]
extern crate rocket;

use prometheus::{Encoder, Gauge, IntCounter, TextEncoder};
use rocket::http::ContentType;
use rocket::Config;
use std::fs;
use std::sync::OnceLock;

static METRICS_REQUESTS_TOTAL: OnceLock<IntCounter> = OnceLock::new();
static UPTIME_SECONDS: OnceLock<Gauge> = OnceLock::new();
static LOAD1: OnceLock<Gauge> = OnceLock::new();
static LOAD5: OnceLock<Gauge> = OnceLock::new();
static LOAD15: OnceLock<Gauge> = OnceLock::new();

fn metrics_requests_total() -> &'static IntCounter {
    METRICS_REQUESTS_TOTAL.get_or_init(|| {
        prometheus::register_int_counter!(
            "metrics_requests_total",
            "Total number of /metrics requests"
        )
        .expect("register metrics_requests_total")
    })
}

fn uptime_seconds() -> &'static Gauge {
    UPTIME_SECONDS.get_or_init(|| {
        prometheus::register_gauge!("uptime_seconds", "System uptime in seconds")
            .expect("register uptime_seconds")
    })
}

fn load1() -> &'static Gauge {
    LOAD1.get_or_init(|| {
        prometheus::register_gauge!("load1", "1-minute load average")
            .expect("register load1")
    })
}

fn load5() -> &'static Gauge {
    LOAD5.get_or_init(|| {
        prometheus::register_gauge!("load5", "5-minute load average")
            .expect("register load5")
    })
}

fn load15() -> &'static Gauge {
    LOAD15.get_or_init(|| {
        prometheus::register_gauge!("load15", "15-minute load average")
            .expect("register load15")
    })
}

fn update_uptime() {
    let Ok(contents) = fs::read_to_string("/proc/uptime") else {
        return;
    };
    let Some(first) = contents.split_whitespace().next() else {
        return;
    };
    if let Ok(value) = first.parse::<f64>() {
        uptime_seconds().set(value);
    }
}

fn update_loadavg() {
    let Ok(contents) = fs::read_to_string("/proc/loadavg") else {
        return;
    };
    let mut parts = contents.split_whitespace();
    let (Some(a), Some(b), Some(c)) = (parts.next(), parts.next(), parts.next()) else {
        return;
    };
    if let Ok(value) = a.parse::<f64>() {
        load1().set(value);
    }
    if let Ok(value) = b.parse::<f64>() {
        load5().set(value);
    }
    if let Ok(value) = c.parse::<f64>() {
        load15().set(value);
    }
}

fn update_metrics() {
    update_uptime();
    update_loadavg();
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
    let figment = Config::figment().merge(("port", 9100));
    rocket::custom(figment).mount("/", routes![index, metrics])
}
