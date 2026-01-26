use gcodekit5_devicedb::{ControllerType, DeviceManager, DeviceProfile, DeviceType};
use tempfile::tempdir;

#[test]
fn test_device_manager_crud() {
    let dir = tempdir().expect("tempdir failed");
    let config_path = dir.path().join("devices.json");
    let manager = DeviceManager::new(config_path.clone());

    // Test Load (should create default)
    manager.load().expect("load failed");
    let profiles = manager.get_all_profiles();
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0].name, "New Device");

    let active = manager.get_active_profile();
    assert!(active.is_some());
    assert_eq!(active.expect("active is None").id, profiles[0].id);

    // Test Create
    let mut new_profile = DeviceProfile::default();
    new_profile.name = "My Laser".to_string();
    new_profile.device_type = DeviceType::LaserCutter;
    manager
        .save_profile(new_profile.clone())
        .expect("save profile failed");

    let profiles = manager.get_all_profiles();
    assert_eq!(profiles.len(), 2);

    // Test Read
    let fetched = manager
        .get_profile(&new_profile.id)
        .expect("profile not found");
    assert_eq!(fetched.name, "My Laser");
    assert_eq!(fetched.device_type, DeviceType::LaserCutter);

    // Test Update
    let mut updated = fetched.clone();
    updated.controller_type = ControllerType::Smoothieware;
    manager.save_profile(updated).expect("save profile failed");

    let fetched_updated = manager
        .get_profile(&new_profile.id)
        .expect("profile not found");
    assert_eq!(
        fetched_updated.controller_type,
        ControllerType::Smoothieware
    );

    // Test Set Active
    manager
        .set_active_profile(&new_profile.id)
        .expect("set active failed");
    let active = manager.get_active_profile().expect("no active profile");
    assert_eq!(active.id, new_profile.id);

    // Test Persistence
    let manager2 = DeviceManager::new(config_path);
    manager2.load().expect("load failed");
    assert_eq!(manager2.get_all_profiles().len(), 2);
    assert_eq!(
        manager2.get_active_profile().expect("no active profile").id,
        new_profile.id
    );

    // Test Delete
    manager2
        .delete_profile(&new_profile.id)
        .expect("delete failed");
    assert_eq!(manager2.get_all_profiles().len(), 1);
    assert!(manager2.get_active_profile().is_none()); // Active profile was deleted
}
