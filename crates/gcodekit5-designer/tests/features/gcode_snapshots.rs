//! Snapshot/golden tests for G-code generation.
//!
//! These tests verify that G-code output from known inputs produces
//! deterministic, expected output. If the output format changes,
//! update the expected strings to match.

use gcodekit5_core::Units;
use gcodekit5_designer::gcode_gen::ToolpathToGcode;
use gcodekit5_designer::model::Point;
use gcodekit5_designer::toolpath::{Toolpath, ToolpathSegment, ToolpathSegmentType};

/// Helper to build a simple rectangular contour toolpath.
fn rectangle_toolpath() -> Toolpath {
    let feed = 500.0;
    let spindle = 12000;
    let mut tp = Toolpath::new(3.175, -2.0);
    // Rapid to start
    tp.add_segment(ToolpathSegment::new(
        ToolpathSegmentType::RapidMove,
        Point::new(0.0, 0.0),
        Point::new(10.0, 10.0),
        feed,
        spindle,
    ));
    // Cut rectangle: 10,10 -> 50,10 -> 50,40 -> 10,40 -> 10,10
    tp.add_segment(ToolpathSegment::new(
        ToolpathSegmentType::LinearMove,
        Point::new(10.0, 10.0),
        Point::new(50.0, 10.0),
        feed,
        spindle,
    ));
    tp.add_segment(ToolpathSegment::new(
        ToolpathSegmentType::LinearMove,
        Point::new(50.0, 10.0),
        Point::new(50.0, 40.0),
        feed,
        spindle,
    ));
    tp.add_segment(ToolpathSegment::new(
        ToolpathSegmentType::LinearMove,
        Point::new(50.0, 40.0),
        Point::new(10.0, 40.0),
        feed,
        spindle,
    ));
    tp.add_segment(ToolpathSegment::new(
        ToolpathSegmentType::LinearMove,
        Point::new(10.0, 40.0),
        Point::new(10.0, 10.0),
        feed,
        spindle,
    ));
    tp
}

/// Helper to build a circle toolpath using arcs.
fn circle_toolpath() -> Toolpath {
    let feed = 400.0;
    let spindle = 10000;
    let center = Point::new(25.0, 25.0);
    let radius = 15.0;
    let mut tp = Toolpath::new(3.0, -1.5);
    // Rapid to start (top of circle)
    tp.add_segment(ToolpathSegment::new(
        ToolpathSegmentType::RapidMove,
        Point::new(0.0, 0.0),
        Point::new(center.x + radius, center.y),
        feed,
        spindle,
    ));
    // Full circle as two CW arcs (top half, bottom half)
    tp.add_segment(ToolpathSegment::new_arc(
        ToolpathSegmentType::ArcCW,
        Point::new(center.x + radius, center.y),
        Point::new(center.x - radius, center.y),
        center,
        feed,
        spindle,
    ));
    tp.add_segment(ToolpathSegment::new_arc(
        ToolpathSegmentType::ArcCW,
        Point::new(center.x - radius, center.y),
        Point::new(center.x + radius, center.y),
        center,
        feed,
        spindle,
    ));
    tp
}

#[test]
fn snapshot_rectangle_contour_3d() {
    let tp = rectangle_toolpath();
    let gen = ToolpathToGcode::new(Units::MM, 5.0);
    let output = gen.generate(&tp);

    // Verify header
    assert!(output.starts_with("; Generated G-code from Designer tool\n"));
    assert!(output.contains("; Tool diameter: 3.175mm"));
    assert!(output.contains("; Cut depth: -2.000mm"));
    assert!(output.contains("; Feed rate: 500 mm/min"));
    assert!(output.contains("; Spindle speed: 12000 RPM"));
    assert!(output.contains("G90"));
    assert!(output.contains("G21"));
    assert!(output.contains("G17"));
    assert!(output.contains("M3 S12000"));

    // Verify body contains expected moves
    assert!(output.contains("G00 X10.000 Y10.000 Z5.000"));
    assert!(output.contains("G01 Z-2.000 F500"));
    assert!(output.contains("G01 X50.000 Y10.000 F500"));
    assert!(output.contains("G01 X50.000 Y40.000 F500"));
    assert!(output.contains("G01 X10.000 Y40.000 F500"));
    assert!(output.contains("G01 X10.000 Y10.000 F500"));

    // Verify footer
    assert!(output.contains("M5"));
    assert!(output.contains("G00 Z5.000"));
    assert!(output.contains("G00 X0 Y0"));
    assert!(output.contains("M30"));
}

#[test]
fn snapshot_rectangle_contour_2d() {
    let tp = rectangle_toolpath();
    let mut gen = ToolpathToGcode::new(Units::MM, 5.0);
    gen.num_axes = 2;
    let output = gen.generate(&tp);

    // 2D mode: no Z coordinates in moves
    assert!(!output.contains("Z5.000"));
    assert!(!output.contains("Z-2.000"));
    // But still has rapid XY moves
    assert!(output.contains("G00 X10.000 Y10.000"));
    assert!(output.contains("G01 X50.000 Y10.000 F500"));
    // Footer should NOT have Z raise
    let footer_idx = output.rfind("M5").unwrap();
    let footer = &output[footer_idx..];
    assert!(!footer.contains("G00 Z"));
}

#[test]
fn snapshot_rectangle_with_line_numbers() {
    let tp = rectangle_toolpath();
    let gen = ToolpathToGcode::with_line_numbers(Units::MM, 5.0, true);
    let output = gen.generate(&tp);

    // Line numbers should appear in body
    assert!(output.contains("N10 "));
    assert!(output.contains("N20 "));
    // Header comments don't get line numbers
    let first_n_pos = output.find("N10 ").unwrap();
    let header_end = output.find("M3 S").unwrap();
    assert!(first_n_pos > header_end);
}

#[test]
fn snapshot_circle_arc_gcode() {
    let tp = circle_toolpath();
    let gen = ToolpathToGcode::new(Units::MM, 5.0);
    let output = gen.generate(&tp);

    // Should contain arc commands with I/J offsets
    assert!(output.contains("G02 X"));
    assert!(output.contains("I-15.000 J0.000"));
    assert!(output.contains("I15.000 J0.000"));
    // Header should reflect circle parameters
    assert!(output.contains("; Tool diameter: 3.000mm"));
    assert!(output.contains("; Cut depth: -1.500mm"));
}

#[test]
fn snapshot_empty_toolpath() {
    let tp = Toolpath::new(3.0, -1.0);
    let gen = ToolpathToGcode::new(Units::MM, 5.0);
    let output = gen.generate(&tp);

    // Should still have header + footer, just no body moves
    assert!(output.contains("; Generated G-code"));
    assert!(output.contains("M30"));
    // No G01 moves
    assert!(!output.contains("G01"));
}

#[test]
fn snapshot_header_format() {
    let gen = ToolpathToGcode::new(Units::MM, 10.0);
    let header = gen.generate_header(18000, 800.0, 6.35, -3.0, 150.0);

    let expected = "\
; Generated G-code from Designer tool
; Tool diameter: 6.350mm
; Cut depth: -3.000mm
; Feed rate: 800 mm/min
; Spindle speed: 18000 RPM
; Total path length: 150.000mm

G90         ; Absolute positioning
G21         ; Millimeter units
G17         ; XY plane
M3 S18000      ; Spindle on at 18000 RPM

";
    assert_eq!(header, expected);
}

#[test]
fn snapshot_footer_format_3d() {
    let gen = ToolpathToGcode::new(Units::MM, 10.0);
    let footer = gen.generate_footer();

    let expected = "\
\nM5          ; Spindle off\n\
G00 Z10.000   ; Raise tool to safe height\n\
G00 X0 Y0   ; Return to origin\n\
M30         ; End program\n";
    assert_eq!(footer, expected);
}

#[test]
fn snapshot_footer_format_2d() {
    let mut gen = ToolpathToGcode::new(Units::MM, 10.0);
    gen.num_axes = 2;
    let footer = gen.generate_footer();

    let expected = "\
\nM5          ; Spindle off\n\
G00 X0 Y0   ; Return to origin\n\
M30         ; End program\n";
    assert_eq!(footer, expected);
}

#[test]
fn snapshot_gcode_deterministic() {
    // Running the same generation twice produces identical output
    let tp = rectangle_toolpath();
    let gen = ToolpathToGcode::new(Units::MM, 5.0);
    let output1 = gen.generate(&tp);
    let output2 = gen.generate(&tp);
    assert_eq!(output1, output2);
}

#[test]
fn snapshot_mixed_segment_types() {
    let mut tp = Toolpath::new(3.0, -1.0);
    let feed = 600.0;
    let spindle = 15000;

    // Rapid to start
    tp.add_segment(ToolpathSegment::new(
        ToolpathSegmentType::RapidMove,
        Point::new(0.0, 0.0),
        Point::new(10.0, 0.0),
        feed,
        spindle,
    ));
    // Linear move
    tp.add_segment(ToolpathSegment::new(
        ToolpathSegmentType::LinearMove,
        Point::new(10.0, 0.0),
        Point::new(20.0, 0.0),
        feed,
        spindle,
    ));
    // CW arc
    tp.add_segment(ToolpathSegment::new_arc(
        ToolpathSegmentType::ArcCW,
        Point::new(20.0, 0.0),
        Point::new(20.0, 10.0),
        Point::new(20.0, 5.0),
        feed,
        spindle,
    ));
    // CCW arc
    tp.add_segment(ToolpathSegment::new_arc(
        ToolpathSegmentType::ArcCCW,
        Point::new(20.0, 10.0),
        Point::new(10.0, 10.0),
        Point::new(15.0, 10.0),
        feed,
        spindle,
    ));

    let gen = ToolpathToGcode::new(Units::MM, 5.0);
    let output = gen.generate(&tp);

    // All move types should be present
    assert!(output.contains("G00"));
    assert!(output.contains("G01"));
    assert!(output.contains("G02"));
    assert!(output.contains("G03"));
}
