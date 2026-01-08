use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

/// Subsystem availability checks
struct SubsystemCheck {
    name: &'static str,
    path: &'static str,
    description: &'static str,
    /// If true, check that directory has entries (not just exists)
    require_entries: bool,
}

const SUBSYSTEM_CHECKS: &[SubsystemCheck] = &[
    SubsystemCheck {
        name: "numa",
        path: "/sys/devices/system/node",
        description: "NUMA",
        require_entries: true,
    },
    SubsystemCheck {
        name: "edac",
        path: "/sys/devices/system/edac/mc",
        description: "EDAC (memory error detection)",
        require_entries: true,
    },
    SubsystemCheck {
        name: "rapl",
        path: "/sys/class/powercap",
        description: "RAPL (power monitoring)",
        require_entries: true,
    },
    SubsystemCheck {
        name: "hwmon",
        path: "/sys/class/hwmon",
        description: "Hardware monitoring",
        require_entries: true,
    },
    SubsystemCheck {
        name: "thermal",
        path: "/sys/class/thermal",
        description: "Thermal zones",
        require_entries: true,
    },
    SubsystemCheck {
        name: "power_supply",
        path: "/sys/class/power_supply",
        description: "Power supply",
        require_entries: true,
    },
    SubsystemCheck {
        name: "nvme",
        path: "/sys/class/nvme",
        description: "NVMe devices",
        require_entries: true,
    },
];

fn check_subsystem_available(check: &SubsystemCheck) -> bool {
    let path = Path::new(check.path);

    if !path.exists() {
        return false;
    }

    if check.require_entries {
        match fs::read_dir(path) {
            Ok(mut entries) => entries.next().is_some(),
            Err(_) => false,
        }
    } else {
        true
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub ignore_loop_devices: bool,
    pub ignore_ppp_interfaces: bool,
    #[serde(default)]
    pub disabled_datasources: Vec<String>,
    #[serde(skip)]
    disabled_set: HashSet<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ignore_loop_devices: true,
            ignore_ppp_interfaces: true,
            disabled_datasources: Vec::new(),
            disabled_set: HashSet::new(),
        }
    }
}

impl AppConfig {
    pub fn is_datasource_enabled(&self, name: &str) -> bool {
        !self.disabled_set.contains(name)
    }

    pub fn disable_datasource(&mut self, name: &str) {
        self.disabled_set.insert(name.to_string());
    }

    fn build_disabled_set(&mut self) {
        self.disabled_set = self.disabled_datasources.iter().cloned().collect();
    }

    pub fn load() -> Self {
        let mut config = match fs::read_to_string("config.toml") {
            Ok(contents) => toml::from_str(&contents).unwrap_or_else(|err| {
                eprintln!("Failed to parse config.toml: {err}");
                Self::default()
            }),
            Err(err) if err.kind() == ErrorKind::NotFound => Self::default(),
            Err(err) => {
                eprintln!("Failed to read config.toml: {err}");
                Self::default()
            }
        };

        config.build_disabled_set();
        config.check_subsystems();
        config
    }

    fn check_subsystems(&mut self) {
        for check in SUBSYSTEM_CHECKS {
            if !self.is_datasource_enabled(check.name) {
                // Already disabled by config, skip check
                continue;
            }

            if !check_subsystem_available(check) {
                eprintln!(
                    "{} subsystem not available ({}), disabling {} datasource.",
                    check.description, check.path, check.name
                );
                self.disable_datasource(check.name);
            }
        }
    }
}
