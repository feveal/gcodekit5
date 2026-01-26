//! Tests for firmware::override_manager

use gcodekit5_communication::firmware::override_manager::*;

#[test]
fn test_feed_rate_override() {
    let mut manager = DefaultOverrideManager::new();
    assert_eq!(manager.get_feed_rate_override(), 100.0);

    manager
        .set_feed_rate_override(150.0)
        .expect("set override failed");
    assert_eq!(manager.get_feed_rate_override(), 150.0);

    assert!(manager.set_feed_rate_override(300.0).is_err());
}

#[test]
fn test_rapid_override() {
    let mut manager = DefaultOverrideManager::new();
    assert_eq!(manager.get_rapid_override(), RapidOverrideLevel::Full);

    manager
        .set_rapid_override(RapidOverrideLevel::Slow)
        .expect("operation failed");
    assert_eq!(manager.get_rapid_override(), RapidOverrideLevel::Slow);
}

#[test]
fn test_spindle_override() {
    let mut manager = DefaultOverrideManager::new();
    assert_eq!(manager.get_spindle_override(), 100.0);

    manager
        .set_spindle_override(75.0)
        .expect("set override failed");
    assert_eq!(manager.get_spindle_override(), 75.0);
}

#[test]
fn test_increase_feed_rate() {
    let mut manager = DefaultOverrideManager::new();
    manager.increase_feed_rate(10.0).expect("increase failed");
    assert_eq!(manager.get_feed_rate_override(), 110.0);
}
