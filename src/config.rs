use std::time::Duration;
use toml::Value;
use dirs::{config_dir, home_dir};

#[derive(Debug, Clone, Copy)]
pub struct AppConfig {
    pub short_break: Option<Duration>,
    pub long_break: Option<Duration>,
    pub long_break_frequency: Option<u32>,
    pub work_time: Option<Duration>,
}

impl AppConfig {
    pub fn load() -> Self {
        let mut config = AppConfig {
            short_break: None,
            long_break: None,
            long_break_frequency: None,
            work_time: None,
        };
        // Try default config_dir first
        let mut tried_paths = vec![];
        let mut found = false;
        if let Some(mut path) = config_dir() {
            path.push("porsmo");
            path.push("porsmo");
            tried_paths.push(path.clone());
            if path.exists() {
                found = true;
                if let Err(_e) = Self::load_from_path(&path, &mut config) {
                    #[cfg(debug_assertions)]
                    eprintln!("[DEBUG] Error loading config from {:?}: {:?}", path, _e);
                }
            }
        }
        // If not found, try ~/.config/porsmo/porsmo
        if !found {
            if let Some(mut home) = home_dir() {
                home.push(".config");
                home.push("porsmo");
                home.push("porsmo");
                tried_paths.push(home.clone());
                if home.exists() {
                    if let Err(_e) = Self::load_from_path(&home, &mut config) {
                        #[cfg(debug_assertions)]
                        eprintln!("[DEBUG] Error loading config from {:?}: {:?}", home, _e);
                    }
                }
            }
        }
        if !found {
            #[cfg(debug_assertions)]
            eprintln!("[DEBUG] Tried config paths: {:?}", tried_paths);
        }
        config
    }

    fn load_from_path(path: &std::path::Path, config: &mut AppConfig) -> Result<(), String> {
        match std::fs::read_to_string(path) {
            Ok(contents) => {
                #[cfg(debug_assertions)]
                eprintln!("[DEBUG] Config file contents from {}:\n{}", path.display(), contents);
                match contents.parse::<Value>() {
                    Ok(toml) => {
                        if let Some(table) = toml.as_table() {
                            if let Some(s) = table.get("short_break_duration").and_then(Value::as_str) {
                                config.short_break = crate::format::parse_duration(s).ok();
                            }
                            if let Some(s) = table.get("long_break_duration").and_then(Value::as_str) {
                                config.long_break = crate::format::parse_duration(s).ok();
                            }
                            if let Some(f) = table.get("long_break_frequency").and_then(Value::as_integer) {
                                config.long_break_frequency = Some(f as u32);
                            }
                            if let Some(s) = table.get("work_time_duration").and_then(Value::as_str) {
                                config.work_time = crate::format::parse_duration(s).ok();
                            }
                        } else {
                            #[cfg(debug_assertions)]
                            eprintln!("[DEBUG] TOML root is not a table");
                            return Err("TOML root is not a table".to_string());
                        }
                    }
                    Err(e) => {
                        #[cfg(debug_assertions)]
                        eprintln!("[DEBUG] TOML parse error: {}", e);
                        return Err(format!("TOML parse error: {}", e));
                    }
                }
            }
            Err(e) => {
                #[cfg(debug_assertions)]
                eprintln!("[DEBUG] Could not read config file: {}", e);
                return Err(format!("Could not read config file: {}", e));
            }
        }
        Ok(())
    }
}
