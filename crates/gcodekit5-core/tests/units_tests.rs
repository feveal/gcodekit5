//! Unit tests for the units conversion module
//!
//! Tests verify correct conversion between Metric (mm) and Imperial (inch) units.

use gcodekit5_core::units::{
    format_feed_rate, format_length, get_unit_label, parse_feed_rate, parse_length, FeedRateUnits,
    MeasurementSystem,
};

const MM_PER_INCH: f32 = 25.4;

#[test]
fn test_format_length_metric() {
    let result = format_length(10.5, MeasurementSystem::Metric);
    assert_eq!(result, "10.500");
}

#[test]
fn test_format_length_imperial() {
    let mm = 25.4; // 1 inch
    let result = format_length(mm, MeasurementSystem::Imperial);
    let parsed: f32 = result.parse().expect("parse failed");
    assert!(
        (parsed - 1.0).abs() < 1e-3,
        "25.4mm should format as ~1 inch"
    );
}

#[test]
fn test_parse_length_metric() {
    let result = parse_length("10.5", MeasurementSystem::Metric).expect("parse_length failed");
    assert_eq!(result, 10.5);
}

#[test]
fn test_parse_length_imperial_decimal() {
    let result = parse_length("1.0", MeasurementSystem::Imperial).expect("parse_length failed");
    assert!(
        (result - 25.4).abs() < 1e-6,
        "1 inch should parse to 25.4mm"
    );
}

#[test]
fn test_parse_length_imperial_fraction() {
    let result = parse_length("1/4", MeasurementSystem::Imperial).expect("parse_length failed");
    let expected = 0.25 * 25.4; // 1/4 inch in mm
    assert!(
        (result - expected).abs() < 1e-3,
        "1/4 inch should parse correctly"
    );
}

#[test]
fn test_parse_length_imperial_mixed() {
    let result = parse_length("1 1/2", MeasurementSystem::Imperial).expect("parse_length failed");
    let expected = 1.5 * 25.4; // 1.5 inches in mm
    assert!(
        (result - expected).abs() < 1e-3,
        "1 1/2 inches should parse correctly"
    );
}

#[test]
fn test_parse_length_empty() {
    let result = parse_length("", MeasurementSystem::Metric).expect("parse_length failed");
    assert_eq!(result, 0.0);
}

#[test]
fn test_format_feed_rate_mm_per_min() {
    let result = format_feed_rate(1000.0, FeedRateUnits::MmPerMin);
    assert_eq!(result, "1000.000");
}

#[test]
fn test_format_feed_rate_mm_per_sec() {
    let result = format_feed_rate(1200.0, FeedRateUnits::MmPerSec);
    let parsed: f32 = result.parse().expect("parse failed");
    assert!(
        (parsed - 20.0).abs() < 1e-3,
        "1200 mm/min should be 20 mm/sec"
    );
}

#[test]
fn test_format_feed_rate_inch_per_min() {
    let result = format_feed_rate(254.0, FeedRateUnits::InPerMin);
    let parsed: f32 = result.parse().expect("parse failed");
    assert!(
        (parsed - 10.0).abs() < 1e-2,
        "254 mm/min should be ~10 in/min"
    );
}

#[test]
fn test_parse_feed_rate_mm_per_min() {
    let result = parse_feed_rate("1000", FeedRateUnits::MmPerMin).expect("parse_feed_rate failed");
    assert_eq!(result, 1000.0);
}

#[test]
fn test_parse_feed_rate_inch_per_min() {
    let result = parse_feed_rate("10", FeedRateUnits::InPerMin).expect("parse_feed_rate failed");
    assert!(
        (result - 254.0).abs() < 1e-3,
        "10 in/min should be 254 mm/min"
    );
}

#[test]
fn test_parse_feed_rate_empty() {
    let result = parse_feed_rate("", FeedRateUnits::MmPerMin).expect("parse_feed_rate failed");
    assert_eq!(result, 0.0);
}

#[test]
fn test_get_unit_label() {
    assert_eq!(get_unit_label(MeasurementSystem::Metric), "mm");
    assert_eq!(get_unit_label(MeasurementSystem::Imperial), "in");
}

#[test]
fn test_mm_to_inch_conversion() {
    let mm = 25.4_f32;
    let inches = mm / MM_PER_INCH;
    assert!(
        (inches - 1.0).abs() < 1e-6,
        "25.4mm should be exactly 1 inch"
    );
}

#[test]
fn test_inch_to_mm_conversion() {
    let inches = 1.0_f32;
    let mm = inches * MM_PER_INCH;
    assert!((mm - 25.4).abs() < 1e-6, "1 inch should be exactly 25.4mm");
}

#[test]
fn test_round_trip_conversion_via_parse_format() {
    let original_mm = 123.456_f32;
    let formatted = format_length(original_mm, MeasurementSystem::Metric);
    let parsed = parse_length(&formatted, MeasurementSystem::Metric).expect("parse_length failed");

    assert!(
        (parsed - original_mm).abs() < 1e-3,
        "Round-trip should preserve value"
    );
}

#[test]
fn test_fractional_inches_parsing() {
    // Common fractional inch values in woodworking/CNC
    let fractions = vec![
        ("1/8", 3.175_f32), // 1/8 inch
        ("1/4", 6.35),      // 1/4 inch
        ("1/2", 12.7),      // 1/2 inch
        ("3/4", 19.05),     // 3/4 inch
    ];

    for (input, expected_mm) in fractions {
        let result = parse_length(input, MeasurementSystem::Imperial).expect("parse_length failed");
        assert!(
            (result - expected_mm).abs() < 0.01,
            "{} inch should be ~{} mm, got {}",
            input,
            expected_mm,
            result
        );
    }
}
