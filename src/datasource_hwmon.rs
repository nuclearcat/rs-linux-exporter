use prometheus::GaugeVec;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

struct HwmonMetrics {
    temperature_celsius: GaugeVec,
    fan_rpm: GaugeVec,
    voltage_volts: GaugeVec,
    power_watts: GaugeVec,
    current_amps: GaugeVec,
}

impl HwmonMetrics {
    fn new() -> Self {
        Self {
            temperature_celsius: prometheus::register_gauge_vec!(
                "hwmon_temperature_celsius",
                "Hardware monitor temperature sensor reading in Celsius",
                &["chip", "sensor"]
            )
            .expect("register hwmon_temperature_celsius"),

            fan_rpm: prometheus::register_gauge_vec!(
                "hwmon_fan_rpm",
                "Hardware monitor fan speed in RPM",
                &["chip", "sensor"]
            )
            .expect("register hwmon_fan_rpm"),

            voltage_volts: prometheus::register_gauge_vec!(
                "hwmon_voltage_volts",
                "Hardware monitor voltage reading in Volts",
                &["chip", "sensor"]
            )
            .expect("register hwmon_voltage_volts"),

            power_watts: prometheus::register_gauge_vec!(
                "hwmon_power_watts",
                "Hardware monitor power reading in Watts",
                &["chip", "sensor"]
            )
            .expect("register hwmon_power_watts"),

            current_amps: prometheus::register_gauge_vec!(
                "hwmon_current_amps",
                "Hardware monitor current reading in Amps",
                &["chip", "sensor"]
            )
            .expect("register hwmon_current_amps"),
        }
    }
}

static HWMON_METRICS: OnceLock<HwmonMetrics> = OnceLock::new();

fn metrics() -> &'static HwmonMetrics {
    HWMON_METRICS.get_or_init(HwmonMetrics::new)
}

fn read_value(path: &Path) -> Option<i64> {
    let contents = fs::read_to_string(path).ok()?;
    contents.trim().parse::<i64>().ok()
}

fn read_string(path: &Path) -> Option<String> {
    let contents = fs::read_to_string(path).ok()?;
    Some(contents.trim().to_string())
}

fn get_sensor_label(hwmon_dir: &Path, sensor_type: &str, index: &str) -> String {
    let label_path = hwmon_dir.join(format!("{}_{}_label", sensor_type, index));
    read_string(&label_path).unwrap_or_else(|| format!("{}_{}", sensor_type, index))
}

fn update_hwmon_device(hwmon_dir: &Path) {
    let chip_name = match read_string(&hwmon_dir.join("name")) {
        Some(name) => name,
        None => return,
    };

    let entries = match fs::read_dir(hwmon_dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    let metrics = metrics();

    for entry in entries.flatten() {
        let file_name = match entry.file_name().into_string() {
            Ok(name) => name,
            Err(_) => continue,
        };

        // Temperature sensors: temp[1-*]_input (millidegrees Celsius)
        if file_name.starts_with("temp") && file_name.ends_with("_input") {
            let index = &file_name[4..file_name.len() - 6];
            if let Some(millidegrees) = read_value(&entry.path()) {
                let label = get_sensor_label(hwmon_dir, "temp", index);
                metrics
                    .temperature_celsius
                    .with_label_values(&[&chip_name, &label])
                    .set(millidegrees as f64 / 1000.0);
            }
        }
        // Fan sensors: fan[1-*]_input (RPM)
        else if file_name.starts_with("fan") && file_name.ends_with("_input") {
            let index = &file_name[3..file_name.len() - 6];
            if let Some(rpm) = read_value(&entry.path()) {
                let label = get_sensor_label(hwmon_dir, "fan", index);
                metrics
                    .fan_rpm
                    .with_label_values(&[&chip_name, &label])
                    .set(rpm as f64);
            }
        }
        // Voltage sensors: in[0-*]_input (millivolts)
        else if file_name.starts_with("in") && file_name.ends_with("_input") {
            let index = &file_name[2..file_name.len() - 6];
            if index.chars().all(|c| c.is_ascii_digit())
                && let Some(millivolts) = read_value(&entry.path())
            {
                let label = get_sensor_label(hwmon_dir, "in", index);
                metrics
                    .voltage_volts
                    .with_label_values(&[&chip_name, &label])
                    .set(millivolts as f64 / 1000.0);
            }
        }
        // Power sensors: power[1-*]_input (microwatts)
        else if file_name.starts_with("power") && file_name.ends_with("_input") {
            let index = &file_name[5..file_name.len() - 6];
            if let Some(microwatts) = read_value(&entry.path()) {
                let label = get_sensor_label(hwmon_dir, "power", index);
                metrics
                    .power_watts
                    .with_label_values(&[&chip_name, &label])
                    .set(microwatts as f64 / 1_000_000.0);
            }
        }
        // Current sensors: curr[1-*]_input (milliamps)
        else if file_name.starts_with("curr") && file_name.ends_with("_input") {
            let index = &file_name[4..file_name.len() - 6];
            if let Some(milliamps) = read_value(&entry.path()) {
                let label = get_sensor_label(hwmon_dir, "curr", index);
                metrics
                    .current_amps
                    .with_label_values(&[&chip_name, &label])
                    .set(milliamps as f64 / 1000.0);
            }
        }
    }
}

pub fn update_metrics() {
    let base = Path::new("/sys/class/hwmon");
    let entries = match fs::read_dir(base) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() || path.is_symlink() {
            // Resolve symlinks to get the actual hwmon directory
            let resolved = match fs::canonicalize(&path) {
                Ok(p) => p,
                Err(_) => continue,
            };
            update_hwmon_device(&resolved);
        }
    }
}
