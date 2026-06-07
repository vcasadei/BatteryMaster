use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use tauri_plugin_autostart::ManagerExt;
#[cfg(debug_assertions)]
fn check_if_dev() -> bool {
    true
}

#[cfg(not(debug_assertions))]
fn check_if_dev() -> bool {
    false
}

// Define configuration struct
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Config {
    pub auto_start: bool,             // Start with the system
    pub start_minimize: bool,         // Start minimized
    pub ui_update: u8,                // UI update interval
    pub service_update: u8,           // Monitoring service update interval
    pub record_battery_history: bool, // Whether to record battery activity history
}

impl Default for Config {
    fn default() -> Self {
        Config {
            auto_start: false,
            start_minimize: false,
            ui_update: 2,
            service_update: 1,
            record_battery_history: true,
        }
    }
}

// Get current executable directory
pub fn get_exe_directory() -> PathBuf {
    env::current_exe()
        .expect("Failed to get current executable path")
        .parent()
        .unwrap()
        .to_path_buf()
}

// Get config file path (based on executable directory)
pub fn get_config_file_path() -> PathBuf {
    get_exe_directory().join("config.json")
}

// Read configuration file and return Config
pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = get_config_file_path();

    if Path::new(&config_path).exists() {
        // File exists: read and parse configuration
        let mut file = File::open(config_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        // Parse JSON configuration
        let config: Config = serde_json::from_str(&contents)?;
        Ok(config)
    } else {
        let config = Config::default();
        save_config(&config).expect("save config err.");
        Ok(config)
    }
}

// Save configuration to file
pub fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_file_path();
    let mut file = File::create(config_path)?;

    // Serialize configuration to JSON
    let config_json = serde_json::to_string_pretty(config)?;
    file.write_all(config_json.as_bytes())?;

    Ok(())
}

// Ensure runtime files exist: history DB and logs file.
pub fn ensure_runtime_files() -> Result<(), Box<dyn std::error::Error>> {
    let exe_dir = get_exe_directory();
    // Ensure directory exists (should normally exist)
    if !exe_dir.exists() {
        std::fs::create_dir_all(&exe_dir)?;
    }

    let history_path = exe_dir.join("history.db");
    if !history_path.exists() {
        let _ = File::create(&history_path)?;
    }

    let logs_path = exe_dir.join("logs.log");
    if !logs_path.exists() {
        let _ = File::create(&logs_path)?;
    }

    Ok(())
}

pub fn set_autostart(app_handle: &tauri::AppHandle, val: bool) {
    let autostart_manager = app_handle.autolaunch();
    if !check_if_dev() {
        if val {
            if !autostart_manager.is_enabled().unwrap() {
                autostart_manager.enable().expect("autostart enable err.")
            }
        } else {
            if autostart_manager.is_enabled().unwrap() {
                autostart_manager.disable().expect("autostart disable err.")
            }
        }
    }
}
