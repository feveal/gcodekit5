//! Cross-crate integration tests: Design → Toolpath → G-code → Visualization
//!
//! Tests the full pipeline from shape creation through G-code generation
//! and visualization parsing.

use gcodekit5_designer::{Circle, DesignerState, Point, Rectangle, Shape};
use gcodekit5_visualizer::{GCodeCommand, GcodeParser, Visualizer};

// ---------------------------------------------------------------------------
// 1. Full workflow: Design → Toolpath → G-code → Visualization
// ---------------------------------------------------------------------------

#[test]
fn full_pipeline_rectangle_contour() {
    let mut state = DesignerState::new();
    state.set_tool_diameter(3.175);
    state.set_feed_rate(500.0);
    state.set_spindle_speed(10000);
    state.set_cut_depth(2.0);

    let rect = Rectangle::new(10.0, 10.0, 40.0, 30.0);
    let _id = state.add_shape_with_undo(Shape::Rectangle(rect));

    let gcode = state.generate_gcode();
    assert!(!gcode.is_empty(), "G-code should not be empty");
    assert!(gcode.contains("G21"), "Should contain metric unit command");

    // Parse with Visualizer
    let mut vis = Visualizer::new();
    vis.parse_gcode(&gcode);
    assert!(
        vis.get_command_count() > 0,
        "Visualizer should have parsed commands"
    );
    assert!(vis.is_dirty(), "Visualizer should be dirty after parsing");

    // Verify bounds encompass the rectangle area
    let (min_x, max_x, min_y, max_y) = vis.get_bounds();
    assert!(max_x > min_x, "X bounds should have non-zero span");
    assert!(max_y > min_y, "Y bounds should have non-zero span");

    // Verify we have movement commands
    let has_moves = vis
        .commands()
        .iter()
        .any(|cmd| matches!(cmd, GCodeCommand::Move { rapid: false, .. }));
    assert!(has_moves, "Should contain cutting moves");
}

#[test]
fn full_pipeline_circle_contour() {
    let mut state = DesignerState::new();
    state.set_tool_diameter(3.175);
    state.set_feed_rate(300.0);
    state.set_spindle_speed(8000);

    let circle = Circle::new(Point::new(50.0, 50.0), 20.0);
    let _id = state.add_shape_with_undo(Shape::Circle(circle));

    let gcode = state.generate_gcode();
    assert!(!gcode.is_empty());

    let mut vis = Visualizer::new();
    vis.parse_gcode(&gcode);
    assert!(vis.get_command_count() > 0);

    // Circles produce arc commands (G2/G3)
    let has_arcs = vis
        .commands()
        .iter()
        .any(|cmd| matches!(cmd, GCodeCommand::Arc { .. }));
    assert!(has_arcs, "Circle contour should produce arc commands");
}

#[test]
fn full_pipeline_multiple_shapes() {
    let mut state = DesignerState::new();
    state.set_tool_diameter(3.0);
    state.set_feed_rate(400.0);
    state.set_spindle_speed(12000);
    state.set_cut_depth(1.5);

    let rect = Rectangle::new(0.0, 0.0, 30.0, 20.0);
    let _id1 = state.add_shape_with_undo(Shape::Rectangle(rect));

    let circle = Circle::new(Point::new(60.0, 30.0), 15.0);
    let _id2 = state.add_shape_with_undo(Shape::Circle(circle));

    assert_eq!(state.canvas.shape_count(), 2);

    let gcode = state.generate_gcode();
    assert!(gcode.contains("Shape ID="), "Should have shape metadata");

    // Count shape comment blocks
    let shape_comments: Vec<&str> = gcode
        .lines()
        .filter(|l| l.starts_with("; Shape ID="))
        .collect();
    assert_eq!(
        shape_comments.len(),
        2,
        "Should have 2 shape comment blocks"
    );

    let mut vis = Visualizer::new();
    vis.parse_gcode(&gcode);

    // Both shapes should produce commands
    let cmd_count = vis.get_command_count();
    assert!(
        cmd_count > 5,
        "Multiple shapes should produce many commands (got {cmd_count})"
    );
}

#[test]
fn pipeline_gcode_roundtrip_consistency() {
    // Generate G-code and verify re-parsing produces same structure
    let mut state = DesignerState::new();
    state.set_tool_diameter(6.0);
    state.set_feed_rate(600.0);
    state.set_spindle_speed(15000);

    let rect = Rectangle::new(5.0, 5.0, 50.0, 50.0);
    let _id = state.add_shape_with_undo(Shape::Rectangle(rect));

    let gcode = state.generate_gcode();

    // Parse with Visualizer twice — second parse should use cache
    let mut vis = Visualizer::new();
    vis.parse_gcode(&gcode);
    let count1 = vis.get_command_count();
    let bounds1 = vis.get_bounds();

    vis.clear_dirty();
    vis.parse_gcode(&gcode);
    let count2 = vis.get_command_count();
    let bounds2 = vis.get_bounds();

    assert_eq!(
        count1, count2,
        "Re-parsing same G-code should yield same command count"
    );
    assert_eq!(bounds1, bounds2, "Bounds should be identical on re-parse");
}

#[test]
fn pipeline_gcode_parser_modal_state_tracks_generated_code() {
    let mut state = DesignerState::new();
    state.set_tool_diameter(3.175);
    state.set_feed_rate(500.0);
    state.set_spindle_speed(10000);

    let rect = Rectangle::new(10.0, 10.0, 30.0, 20.0);
    let _id = state.add_shape_with_undo(Shape::Rectangle(rect));

    let gcode = state.generate_gcode();

    // Parse line-by-line with GcodeParser and verify modal state
    let mut parser = GcodeParser::new();
    let mut parsed_commands = 0;
    let mut errors = 0;

    for line in gcode.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('(') {
            continue;
        }
        match parser.parse(trimmed) {
            Ok(_cmd) => parsed_commands += 1,
            Err(_e) => errors += 1,
        }
    }

    assert!(parsed_commands > 0, "Should parse some commands");
    // Generated G-code should be valid — allow small number of comment-like errors
    let error_ratio = errors as f64 / (parsed_commands + errors) as f64;
    assert!(
        error_ratio < 0.1,
        "Error ratio should be < 10% (got {errors}/{} = {:.1}%)",
        parsed_commands + errors,
        error_ratio * 100.0
    );

    // State should reflect the generated code
    let gcode_state = parser.get_state();
    assert!(
        gcode_state.feed_rate > 0.0,
        "Feed rate should be set after parsing generated G-code"
    );
}

// ---------------------------------------------------------------------------
// 2. SVG output verification
// ---------------------------------------------------------------------------

#[test]
fn pipeline_svg_output_not_empty_after_parsing() {
    let mut state = DesignerState::new();
    state.set_tool_diameter(3.175);
    state.set_feed_rate(500.0);
    state.set_spindle_speed(10000);

    let rect = Rectangle::new(10.0, 10.0, 30.0, 20.0);
    let _id = state.add_shape_with_undo(Shape::Rectangle(rect));

    let gcode = state.generate_gcode();
    let mut vis = Visualizer::new();
    vis.parse_gcode(&gcode);

    // SVG paths should contain move/line data
    let toolpath_svg = vis.toolpath_svg();
    // At minimum the rapid or G1 SVG should have content
    let g1_svg = vis.g1_svg();
    let any_svg_content = !toolpath_svg.is_empty() || !g1_svg.is_empty();
    assert!(
        any_svg_content,
        "Should have SVG path data after parsing G-code"
    );
}

// ---------------------------------------------------------------------------
// 3. Large file handling: 10K+ line G-code
// ---------------------------------------------------------------------------

#[test]
fn large_gcode_file_parsing() {
    // Generate a large G-code file manually (10K+ lines)
    let mut gcode = String::with_capacity(500_000);
    gcode.push_str("G21\n"); // metric
    gcode.push_str("G90\n"); // absolute
    gcode.push_str("G0 Z5.000\n");
    gcode.push_str("M3 S10000\n");

    // Generate a spiral pattern with 10000+ lines
    for i in 0..10_000 {
        let angle = (i as f64) * 0.1;
        let radius = 1.0 + (i as f64) * 0.005;
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        let z = -0.01 * (i as f64 / 100.0);
        gcode.push_str(&format!("G1 X{:.3} Y{:.3} Z{:.3} F500\n", x, y, z));
    }

    gcode.push_str("G0 Z5.000\n");
    gcode.push_str("M5\n");
    gcode.push_str("M2\n");

    let line_count = gcode.lines().count();
    assert!(
        line_count > 10_000,
        "Generated G-code should be 10K+ lines (got {line_count})"
    );

    // Parse with Visualizer
    let mut vis = Visualizer::new();
    vis.parse_gcode(&gcode);
    assert!(
        vis.get_command_count() > 9_000,
        "Should parse most commands from large file (got {})",
        vis.get_command_count()
    );

    // Parse with GcodeParser line-by-line
    let mut parser = GcodeParser::new();
    let mut parsed = 0;
    for line in gcode.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with(';') {
            continue;
        }
        if parser.parse(trimmed).is_ok() {
            parsed += 1;
        }
    }
    assert!(
        parsed > 9_000,
        "Line-by-line parser should handle large file (got {parsed})"
    );
}

#[test]
fn large_gcode_caching_behavior() {
    let mut gcode = String::with_capacity(200_000);
    gcode.push_str("G21\nG90\n");
    for i in 0..5_000 {
        gcode.push_str(&format!(
            "G1 X{:.1} Y{:.1} F300\n",
            i as f64 * 0.1,
            i as f64 * 0.05
        ));
    }

    let mut vis = Visualizer::new();
    vis.parse_gcode(&gcode);
    let count1 = vis.get_command_count();
    vis.clear_dirty();

    // Re-parse same content — should use cache (not re-parse)
    vis.parse_gcode(&gcode);
    assert!(
        !vis.is_dirty(),
        "Re-parsing identical content should use cache"
    );
    assert_eq!(
        vis.get_command_count(),
        count1,
        "Cache should preserve command count"
    );
}

// ---------------------------------------------------------------------------
// 4. Error recovery / resilience
// ---------------------------------------------------------------------------

#[test]
fn parser_handles_corrupted_gcode_lines() {
    let corrupted_gcode = "\
G21
G90
G0 X10 Y10
G1 X20 Y20 F500
THIS IS NOT GCODE
G1 X30 Y30
XYZZY PLUGH
G1 X40 Y40
G0 Z5
M5
M2
";

    let mut parser = GcodeParser::new();
    let mut total = 0;

    for line in corrupted_gcode.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Parser should not panic on any input
        let _ = parser.parse(trimmed);
        total += 1;
    }

    assert!(total >= 10, "Should have processed all lines without panic");

    // Parser state should be valid after processing mixed input
    let gcode_state = parser.get_state();
    assert!(
        gcode_state.feed_rate > 0.0,
        "Feed rate should be set from valid G1 lines"
    );
}

#[test]
fn parser_handles_truncated_gcode() {
    // Simulate a file that was cut short mid-line
    let truncated = "G21\nG90\nG0 X10 Y10 Z5\nG1 X20 Y20 F500\nG1 X30 Y";

    let mut parser = GcodeParser::new();
    let mut parsed = 0;
    for line in truncated.lines() {
        if parser.parse(line.trim()).is_ok() {
            parsed += 1;
        }
    }
    // Should parse at least the complete lines
    assert!(
        parsed >= 3,
        "Should parse complete lines even with truncated ending"
    );
}

#[test]
fn visualizer_handles_empty_gcode() {
    let mut vis = Visualizer::new();
    vis.parse_gcode("");
    assert_eq!(
        vis.get_command_count(),
        0,
        "Empty G-code should produce no commands"
    );
}

#[test]
fn visualizer_handles_comments_only() {
    let comments = "; This is a comment\n; Another comment\n(parenthetical comment)\n";
    let mut vis = Visualizer::new();
    vis.parse_gcode(comments);
    assert_eq!(
        vis.get_command_count(),
        0,
        "Comments-only G-code should produce no commands"
    );
}

// ---------------------------------------------------------------------------
// 5. Concurrent / parallel operations
// ---------------------------------------------------------------------------

#[test]
fn concurrent_visualizer_instances() {
    // Multiple Visualizer instances should work independently
    let gcode_a = "G21\nG90\nG0 X10 Y10\nG1 X20 Y20 F500\n";
    let gcode_b = "G21\nG90\nG0 X100 Y100\nG1 X200 Y200 F1000\nG1 X300 Y300\n";

    let mut vis_a = Visualizer::new();
    let mut vis_b = Visualizer::new();

    vis_a.parse_gcode(gcode_a);
    vis_b.parse_gcode(gcode_b);

    // Each should have its own independent state
    assert_ne!(
        vis_a.get_command_count(),
        vis_b.get_command_count(),
        "Independent visualizers should have different command counts"
    );

    let (_, max_x_a, _, _) = vis_a.get_bounds();
    let (_, max_x_b, _, _) = vis_b.get_bounds();
    assert!(
        max_x_b > max_x_a,
        "Visualizer B should have larger X bounds"
    );
}

#[test]
fn concurrent_designer_states() {
    // Multiple DesignerState instances should be fully independent
    let mut state_a = DesignerState::new();
    let mut state_b = DesignerState::new();

    state_a.set_tool_diameter(3.0);
    state_b.set_tool_diameter(6.0);

    let rect = Rectangle::new(0.0, 0.0, 20.0, 20.0);
    let _id_a = state_a.add_shape_with_undo(Shape::Rectangle(rect.clone()));
    assert_eq!(state_a.canvas.shape_count(), 1);
    assert_eq!(
        state_b.canvas.shape_count(),
        0,
        "State B should be independent"
    );

    let circle = Circle::new(Point::new(50.0, 50.0), 10.0);
    let _id_b1 = state_b.add_shape_with_undo(Shape::Circle(circle.clone()));
    let _id_b2 =
        state_b.add_shape_with_undo(Shape::Circle(Circle::new(Point::new(80.0, 80.0), 5.0)));
    assert_eq!(state_b.canvas.shape_count(), 2);
    assert_eq!(
        state_a.canvas.shape_count(),
        1,
        "State A should be unaffected"
    );

    // Generate G-code independently
    let gcode_a = state_a.generate_gcode();
    let gcode_b = state_b.generate_gcode();
    assert_ne!(
        gcode_a, gcode_b,
        "Different designs should produce different G-code"
    );
}

#[test]
fn parallel_thread_parsing() {
    use std::thread;

    let gcode1 = "G21\nG90\nG0 X10 Y10\nG1 X20 Y20 F500\nG1 X30 Y30\n".to_string();
    let gcode2 = "G20\nG90\nG0 X1 Y1\nG1 X2 Y2 F20\nG1 X3 Y3\nG1 X4 Y4\n".to_string();

    let handle1 = thread::spawn(move || {
        let mut vis = Visualizer::new();
        vis.parse_gcode(&gcode1);
        vis.get_command_count()
    });

    let handle2 = thread::spawn(move || {
        let mut vis = Visualizer::new();
        vis.parse_gcode(&gcode2);
        vis.get_command_count()
    });

    let count1 = handle1.join().expect("Thread 1 should complete");
    let count2 = handle2.join().expect("Thread 2 should complete");

    assert!(count1 > 0, "Thread 1 should parse commands");
    assert!(count2 > 0, "Thread 2 should parse commands");
}
