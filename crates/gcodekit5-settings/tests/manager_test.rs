use gcodekit5_settings::{Config, ConnectionType, SettingsManager};
use std::path::PathBuf;

#[test]
fn test_settings_manager_new() {
    let mgr = SettingsManager::new();
    assert!(mgr.config().validate().is_ok());
}

#[test]
fn test_settings_manager_with_config() {
    let mut config = Config::new();
    config.connection.baud_rate = 9600;
    let mgr = SettingsManager::with_config(config);
    assert_eq!(mgr.config().connection.baud_rate, 9600);
}

#[test]
fn test_settings_manager_config_mut() {
    let mut mgr = SettingsManager::new();
    mgr.config_mut().connection.baud_rate = 57600;
    assert_eq!(mgr.config().connection.baud_rate, 57600);
}

#[test]
fn test_settings_manager_save_load() {
    let dir = std::env::temp_dir().join("gcodekit5_test_mgr");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("mgr_config.json");

    let mut mgr = SettingsManager::new();
    mgr.config_mut().connection.baud_rate = 38400;
    mgr.save_to_file(&path).unwrap();

    let loaded = SettingsManager::load_from_file(&path).unwrap();
    assert_eq!(loaded.config().connection.baud_rate, 38400);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_default_grbl_settings() {
    let config = SettingsManager::default_grbl_settings();
    assert_eq!(config.connection.baud_rate, 115200);
    assert!(config.validate().is_ok());
}

#[test]
fn test_default_tinyg_settings() {
    let config = SettingsManager::default_tinyg_settings();
    assert_eq!(config.connection.baud_rate, 115200);
    assert_eq!(config.machine.x_limit, 250.0);
    assert!(config.validate().is_ok());
}

#[test]
fn test_default_g2core_settings() {
    let config = SettingsManager::default_g2core_settings();
    assert_eq!(config.connection.connection_type, ConnectionType::Tcp);
    assert!(config.validate().is_ok());
}

#[test]
fn test_default_smoothieware_settings() {
    let config = SettingsManager::default_smoothieware_settings();
    assert!(config.validate().is_ok());
}

#[test]
fn test_default_fluidnc_settings() {
    let config = SettingsManager::default_fluidnc_settings();
    assert_eq!(config.connection.connection_type, ConnectionType::WebSocket);
    assert!(config.validate().is_ok());
}

#[test]
fn test_all_firmware_defaults_valid() {
    let configs = vec![
        SettingsManager::default_grbl_settings(),
        SettingsManager::default_tinyg_settings(),
        SettingsManager::default_g2core_settings(),
        SettingsManager::default_smoothieware_settings(),
        SettingsManager::default_fluidnc_settings(),
    ];
    for (i, config) in configs.iter().enumerate() {
        assert!(
            config.validate().is_ok(),
            "Firmware config {} failed validation",
            i
        );
    }
}

#[test]
fn test_config_directory_returns_path() {
    let result = SettingsManager::config_directory();
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(path.to_string_lossy().contains("gcodekit5"));
}

#[test]
fn test_config_file_path() {
    let result = SettingsManager::config_file_path();
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(path.to_string_lossy().ends_with("config.json"));
}

#[test]
fn test_recent_files_roundtrip() {
    let dir = std::env::temp_dir().join("gcodekit5_test_recent");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("recent_config.json");

    let mut config = Config::new();
    config.add_recent_file(PathBuf::from("/tmp/file1.gcode"));
    config.add_recent_file(PathBuf::from("/tmp/file2.gcode"));
    config.save_to_file(&path).unwrap();

    let loaded = Config::load_from_file(&path).unwrap();
    assert_eq!(loaded.recent_files.len(), 2);

    std::fs::remove_dir_all(&dir).ok();
}
