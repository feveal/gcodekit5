use gcodekit5_settings::Config;
use std::path::PathBuf;

#[test]
fn test_config_defaults_are_valid() {
    let config = Config::new();
    assert!(config.validate().is_ok());
}

#[test]
fn test_config_json_round_trip() {
    let dir = std::env::temp_dir().join("gcodekit5_test_settings");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("test_config.json");

    let mut config = Config::new();
    config.connection.baud_rate = 9600;
    config.ui.font_size = 14;
    config.save_to_file(&path).unwrap();

    let loaded = Config::load_from_file(&path).unwrap();
    assert_eq!(loaded.connection.baud_rate, 9600);
    assert_eq!(loaded.ui.font_size, 14);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_config_toml_round_trip() {
    let dir = std::env::temp_dir().join("gcodekit5_test_settings_toml");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("test_config.toml");

    let config = Config::new();
    config.save_to_file(&path).unwrap();

    let loaded = Config::load_from_file(&path).unwrap();
    assert_eq!(loaded.connection.baud_rate, config.connection.baud_rate);
    assert_eq!(loaded.ui.window_width, config.ui.window_width);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_config_unsupported_format() {
    let path = std::path::Path::new("/tmp/test.yaml");
    let config = Config::new();
    assert!(config.save_to_file(path).is_err());
}

#[test]
fn test_config_validate_zero_baud_rate() {
    let mut config = Config::new();
    config.connection.baud_rate = 0;
    assert!(config.validate().is_err());
}

#[test]
fn test_config_validate_zero_timeout() {
    let mut config = Config::new();
    config.connection.timeout_ms = 0;
    assert!(config.validate().is_err());
}

#[test]
fn test_config_validate_zero_window_dims() {
    let mut config = Config::new();
    config.ui.window_width = 0;
    assert!(config.validate().is_err());
}

#[test]
fn test_config_validate_zero_font_size() {
    let mut config = Config::new();
    config.ui.font_size = 0;
    assert!(config.validate().is_err());
}

#[test]
fn test_config_validate_zero_arc_segment() {
    let mut config = Config::new();
    config.file_processing.arc_segment_length = 0.0;
    assert!(config.validate().is_err());
}

#[test]
fn test_config_validate_zero_machine_limits() {
    let mut config = Config::new();
    config.machine.x_limit = 0.0;
    assert!(config.validate().is_err());
}

#[test]
fn test_config_validate_zero_jog_feed_rate() {
    let mut config = Config::new();
    config.machine.jog_feed_rate = 0.0;
    assert!(config.validate().is_err());
}

#[test]
fn test_recent_files_add() {
    let mut config = Config::new();
    config.add_recent_file(PathBuf::from("/tmp/a.gcode"));
    config.add_recent_file(PathBuf::from("/tmp/b.gcode"));
    assert_eq!(config.recent_files.len(), 2);
    assert_eq!(config.recent_files[0], PathBuf::from("/tmp/b.gcode"));
}

#[test]
fn test_recent_files_dedup() {
    let mut config = Config::new();
    config.add_recent_file(PathBuf::from("/tmp/a.gcode"));
    config.add_recent_file(PathBuf::from("/tmp/b.gcode"));
    config.add_recent_file(PathBuf::from("/tmp/a.gcode"));
    assert_eq!(config.recent_files.len(), 2);
    assert_eq!(config.recent_files[0], PathBuf::from("/tmp/a.gcode"));
}

#[test]
fn test_recent_files_truncate() {
    let mut config = Config::new();
    let max = config.file_processing.recent_files_count;
    for i in 0..max + 5 {
        config.add_recent_file(PathBuf::from(format!("/tmp/{}.gcode", i)));
    }
    assert_eq!(config.recent_files.len(), max);
}

#[test]
fn test_config_merge_theme() {
    let mut base = Config::new();
    let mut other = Config::new();
    other.ui.theme = gcodekit5_settings::Theme::Dark;
    other.ui.font_size = 18;
    base.merge(&other);
    assert_eq!(base.ui.theme, gcodekit5_settings::Theme::Dark);
    assert_eq!(base.ui.font_size, 18);
}

#[test]
fn test_connection_type_display() {
    assert_eq!(
        gcodekit5_settings::ConnectionType::Serial.to_string(),
        "serial"
    );
    assert_eq!(gcodekit5_settings::ConnectionType::Tcp.to_string(), "tcp");
    assert_eq!(
        gcodekit5_settings::ConnectionType::WebSocket.to_string(),
        "websocket"
    );
}
