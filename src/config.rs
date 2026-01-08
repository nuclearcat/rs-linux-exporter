use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::io::ErrorKind;

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

    fn build_disabled_set(&mut self) {
        self.disabled_set = self.disabled_datasources.iter().cloned().collect();
    }

    pub fn load() -> Self {
        let contents = match fs::read_to_string("config.toml") {
            Ok(contents) => contents,
            Err(err) if err.kind() == ErrorKind::NotFound => return Self::default(),
            Err(err) => {
                eprintln!("Failed to read config.toml: {err}");
                return Self::default();
            }
        };

        let mut config: Self = toml::from_str(&contents).unwrap_or_else(|err| {
            eprintln!("Failed to parse config.toml: {err}");
            Self::default()
        });
        config.build_disabled_set();
        config
    }
}
