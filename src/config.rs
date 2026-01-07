use serde::Deserialize;
use std::fs;
use std::io::ErrorKind;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub ignore_loop_devices: bool,
    pub ignore_ppp_interfaces: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ignore_loop_devices: true,
            ignore_ppp_interfaces: true,
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        let contents = match fs::read_to_string("config.toml") {
            Ok(contents) => contents,
            Err(err) if err.kind() == ErrorKind::NotFound => return Self::default(),
            Err(err) => {
                eprintln!("Failed to read config.toml: {err}");
                return Self::default();
            }
        };

        toml::from_str(&contents).unwrap_or_else(|err| {
            eprintln!("Failed to parse config.toml: {err}");
            Self::default()
        })
    }
}
