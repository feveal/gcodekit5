use gcodekit5_devicedb::{
    ControllerType, DeviceManager, DeviceProfile, DeviceProfileUiModel, DeviceType,
};
use std::path::PathBuf;

fn temp_path(name: &str) -> PathBuf {
    std::env::temp_dir()
        .join("gcodekit5_devicedb_tests")
        .join(name)
}

fn setup_manager(name: &str) -> DeviceManager {
    let dir = temp_path(name);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("devices.json");
    // Clean up from previous test run
    let _ = std::fs::remove_file(&path);
    DeviceManager::new(path)
}

fn cleanup(name: &str) {
    let _ = std::fs::remove_dir_all(temp_path(name));
}

// --- DeviceProfile tests ---

#[test]
fn test_device_profile_defaults() {
    let profile = DeviceProfile::default();
    assert_eq!(profile.name, "New Device");
    assert_eq!(profile.device_type, DeviceType::CncMill);
    assert_eq!(profile.controller_type, ControllerType::Grbl);
    assert_eq!(profile.num_axes, 3);
    assert!(profile.has_spindle);
    assert!(!profile.has_laser);
    assert_eq!(profile.baud_rate, 115200);
}

#[test]
fn test_device_profile_serialization() {
    let profile = DeviceProfile::default();
    let json = serde_json::to_string(&profile).unwrap();
    let deser: DeviceProfile = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.name, profile.name);
    assert_eq!(deser.device_type, profile.device_type);
    assert_eq!(deser.num_axes, profile.num_axes);
}

#[test]
fn test_device_type_display() {
    assert_eq!(DeviceType::CncMill.to_string(), "CNC Mill");
    assert_eq!(DeviceType::CncLathe.to_string(), "CNC Lathe");
    assert_eq!(DeviceType::LaserCutter.to_string(), "Laser Cutter");
    assert_eq!(DeviceType::ThreeDPrinter.to_string(), "3D Printer");
    assert_eq!(DeviceType::Plotter.to_string(), "Plotter");
}

#[test]
fn test_controller_type_display() {
    assert_eq!(ControllerType::Grbl.to_string(), "GRBL");
    assert_eq!(ControllerType::TinyG.to_string(), "TinyG");
    assert_eq!(ControllerType::FluidNC.to_string(), "FluidNC");
    assert_eq!(ControllerType::Marlin.to_string(), "Marlin");
}

// --- DeviceManager tests ---

#[test]
fn test_manager_load_creates_default() {
    let mgr = setup_manager("load_default");
    mgr.load().unwrap();
    let profiles = mgr.get_all_profiles();
    assert_eq!(profiles.len(), 1);
    assert!(mgr.get_active_profile().is_some());
    cleanup("load_default");
}

#[test]
fn test_manager_save_and_reload() {
    let dir = temp_path("save_reload");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("devices.json");
    let _ = std::fs::remove_file(&path);

    let mgr = DeviceManager::new(path.clone());
    mgr.load().unwrap();

    let mut profile = DeviceProfile::default();
    profile.id = "test-profile-1".to_string();
    profile.name = "My CNC".to_string();
    mgr.save_profile(profile).unwrap();

    // Reload from disk
    let mgr2 = DeviceManager::new(path);
    mgr2.load().unwrap();
    let p = mgr2.get_profile("test-profile-1");
    assert!(p.is_some());
    assert_eq!(p.unwrap().name, "My CNC");
    cleanup("save_reload");
}

#[test]
fn test_manager_delete_profile() {
    let mgr = setup_manager("delete");
    mgr.load().unwrap();

    let mut profile = DeviceProfile::default();
    profile.id = "to-delete".to_string();
    profile.name = "Delete Me".to_string();
    mgr.save_profile(profile).unwrap();

    assert!(mgr.get_profile("to-delete").is_some());
    mgr.delete_profile("to-delete").unwrap();
    assert!(mgr.get_profile("to-delete").is_none());
    cleanup("delete");
}

#[test]
fn test_manager_active_profile_cleared_on_delete() {
    let mgr = setup_manager("active_clear");
    mgr.load().unwrap();

    let mut profile = DeviceProfile::default();
    profile.id = "active-del".to_string();
    mgr.save_profile(profile).unwrap();
    mgr.set_active_profile("active-del").unwrap();
    assert!(mgr.get_active_profile().is_some());

    mgr.delete_profile("active-del").unwrap();
    assert!(mgr.get_active_profile().is_none());
    cleanup("active_clear");
}

#[test]
fn test_manager_set_active_nonexistent() {
    let mgr = setup_manager("nonexistent");
    mgr.load().unwrap();
    let result = mgr.set_active_profile("does-not-exist");
    assert!(result.is_err());
    cleanup("nonexistent");
}

#[test]
fn test_manager_get_profile_not_found() {
    let mgr = setup_manager("not_found");
    mgr.load().unwrap();
    assert!(mgr.get_profile("nope").is_none());
    cleanup("not_found");
}

#[test]
fn test_manager_multiple_profiles() {
    let mgr = setup_manager("multi");
    mgr.load().unwrap();

    for i in 0..5 {
        let mut p = DeviceProfile::default();
        p.id = format!("profile-{}", i);
        p.name = format!("Machine {}", i);
        mgr.save_profile(p).unwrap();
    }

    let all = mgr.get_all_profiles();
    // Default profile + 5 added
    assert!(all.len() >= 5);
    cleanup("multi");
}

// --- DeviceProfileUiModel tests ---

#[test]
fn test_ui_model_from_profile() {
    let mut profile = DeviceProfile::default();
    profile.name = "Test CNC".to_string();
    profile.device_type = DeviceType::LaserCutter;
    profile.max_feed_rate = 2000.0;

    let ui_model: DeviceProfileUiModel = profile.into();
    assert_eq!(ui_model.name, "Test CNC");
    assert_eq!(ui_model.device_type, "Laser Cutter");
    assert_eq!(ui_model.max_feed_rate, "2000");
    assert!(!ui_model.is_active);
}

#[test]
fn test_ui_model_axis_formatting() {
    let mut profile = DeviceProfile::default();
    profile.x_axis.min = 0.0;
    profile.x_axis.max = 300.5;

    let ui_model: DeviceProfileUiModel = profile.into();
    assert_eq!(ui_model.x_min, "0.00");
    assert_eq!(ui_model.x_max, "300.50");
}

#[test]
fn test_profile_grbl_settings() {
    let mut profile = DeviceProfile::default();
    profile.grbl_settings.insert(110, "1000.000".to_string());
    profile.grbl_settings.insert(111, "1000.000".to_string());

    let json = serde_json::to_string(&profile).unwrap();
    let deser: DeviceProfile = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.grbl_settings.len(), 2);
    assert_eq!(deser.grbl_settings.get(&110).unwrap(), "1000.000");
}
