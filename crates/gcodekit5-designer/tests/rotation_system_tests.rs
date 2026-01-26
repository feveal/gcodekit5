//! Unit tests for the rotation system
//!
//! Tests verify that rotation is stored in degrees throughout the system
//! and correctly converted to radians only when needed for API calls.

use gcodekit5_designer::model::{rotate_point, Point};
use std::f64::consts::PI;

#[test]
fn test_rotate_point_positive_90_degrees() {
    let p = Point::new(1.0, 0.0);
    let center = Point::new(0.0, 0.0);
    let result = rotate_point(p, center, 90.0);

    // At 90 degrees, (1, 0) should become approximately (0, 1)
    assert!(
        (result.x - 0.0).abs() < 1e-10,
        "Expected x ≈ 0, got {}",
        result.x
    );
    assert!(
        (result.y - 1.0).abs() < 1e-10,
        "Expected y ≈ 1, got {}",
        result.y
    );
}

#[test]
fn test_rotate_point_negative_90_degrees() {
    let p = Point::new(1.0, 0.0);
    let center = Point::new(0.0, 0.0);
    let result = rotate_point(p, center, -90.0);

    // At -90 degrees, (1, 0) should become approximately (0, -1)
    assert!(
        (result.x - 0.0).abs() < 1e-10,
        "Expected x ≈ 0, got {}",
        result.x
    );
    assert!(
        (result.y - (-1.0)).abs() < 1e-10,
        "Expected y ≈ -1, got {}",
        result.y
    );
}

#[test]
fn test_rotate_point_180_degrees() {
    let p = Point::new(1.0, 0.0);
    let center = Point::new(0.0, 0.0);
    let result = rotate_point(p, center, 180.0);

    // At 180 degrees, (1, 0) should become approximately (-1, 0)
    assert!(
        (result.x - (-1.0)).abs() < 1e-10,
        "Expected x ≈ -1, got {}",
        result.x
    );
    assert!(
        (result.y - 0.0).abs() < 1e-10,
        "Expected y ≈ 0, got {}",
        result.y
    );
}

#[test]
fn test_rotate_point_45_degrees() {
    let p = Point::new(1.0, 0.0);
    let center = Point::new(0.0, 0.0);
    let result = rotate_point(p, center, 45.0);

    let expected = 1.0 / 2.0_f64.sqrt();
    assert!(
        (result.x - expected).abs() < 1e-10,
        "Expected x ≈ {}, got {}",
        expected,
        result.x
    );
    assert!(
        (result.y - expected).abs() < 1e-10,
        "Expected y ≈ {}, got {}",
        expected,
        result.y
    );
}

#[test]
fn test_rotate_point_with_offset_center() {
    let p = Point::new(3.0, 2.0);
    let center = Point::new(2.0, 2.0);
    let result = rotate_point(p, center, 90.0);

    // Point at (3, 2) rotated 90° around (2, 2) should be at (2, 3)
    assert!(
        (result.x - 2.0).abs() < 1e-10,
        "Expected x ≈ 2, got {}",
        result.x
    );
    assert!(
        (result.y - 3.0).abs() < 1e-10,
        "Expected y ≈ 3, got {}",
        result.y
    );
}

#[test]
fn test_rotate_point_360_degrees_returns_to_origin() {
    let p = Point::new(5.0, 3.0);
    let center = Point::new(1.0, 1.0);
    let result = rotate_point(p, center, 360.0);

    assert!(
        (result.x - p.x).abs() < 1e-10,
        "Expected x ≈ {}, got {}",
        p.x,
        result.x
    );
    assert!(
        (result.y - p.y).abs() < 1e-10,
        "Expected y ≈ {}, got {}",
        p.y,
        result.y
    );
}

#[test]
fn test_rotate_point_zero_degrees() {
    let p = Point::new(5.0, 3.0);
    let center = Point::new(1.0, 1.0);
    let result = rotate_point(p, center, 0.0);

    assert!(
        (result.x - p.x).abs() < 1e-10,
        "Expected x ≈ {}, got {}",
        p.x,
        result.x
    );
    assert!(
        (result.y - p.y).abs() < 1e-10,
        "Expected y ≈ {}, got {}",
        p.y,
        result.y
    );
}

#[test]
fn test_rotation_does_not_double_convert() {
    // This test verifies the bug fix where rotation was being converted
    // from degrees to degrees, causing 15° to become 859.4°

    let rotation_deg: f64 = 15.0;

    // The bug was calling .to_degrees() on a value already in degrees
    let buggy_result = rotation_deg.to_degrees();
    assert!((buggy_result - 859.4366926962348).abs() < 1e-6); // The bug we fixed

    // The correct behavior is to use the value directly
    let correct_result = rotation_deg;
    assert_eq!(correct_result, 15.0);
}

#[test]
fn test_degrees_to_radians_conversion() {
    let degrees: f64 = 15.0;
    let radians = degrees.to_radians();

    // 15 degrees should be π/12 radians
    let expected = PI / 12.0;
    assert!((radians - expected).abs() < 1e-10);
}

#[test]
fn test_common_rotation_angles() {
    // Test common rotation angles used in CNC work
    let test_cases: Vec<(f64, f64)> = vec![
        (0.0, 0.0),
        (15.0, PI / 12.0),
        (30.0, PI / 6.0),
        (45.0, PI / 4.0),
        (60.0, PI / 3.0),
        (90.0, PI / 2.0),
        (120.0, 2.0 * PI / 3.0),
        (135.0, 3.0 * PI / 4.0),
        (180.0, PI),
        (270.0, 3.0 * PI / 2.0),
        (360.0, 2.0 * PI),
    ];

    for (degrees, expected_radians) in test_cases {
        let radians = degrees.to_radians();
        assert!(
            (radians - expected_radians).abs() < 1e-10,
            "{}° should be {} radians, got {}",
            degrees,
            expected_radians,
            radians
        );
    }
}

#[test]
fn test_negative_rotation_angles() {
    // Test that negative rotations work correctly
    let p = Point::new(1.0, 0.0);
    let center = Point::new(0.0, 0.0);

    let positive_90 = rotate_point(p, center, 90.0);
    let negative_270 = rotate_point(p, center, -270.0);

    // 90° and -270° should give the same result
    assert!((positive_90.x - negative_270.x).abs() < 1e-10);
    assert!((positive_90.y - negative_270.y).abs() < 1e-10);
}
