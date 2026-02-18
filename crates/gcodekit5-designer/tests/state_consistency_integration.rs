//! State consistency integration tests: undo/redo with multi-step operations

use gcodekit5_designer::{Circle, DesignerState, Point, Rectangle, Shape};

#[test]
fn undo_redo_add_single_shape() {
    let mut state = DesignerState::new();
    assert_eq!(state.canvas.shape_count(), 0);
    assert!(!state.can_undo());

    let rect = Rectangle::new(10.0, 10.0, 30.0, 20.0);
    let id = state.add_shape_with_undo(Shape::Rectangle(rect));

    assert_eq!(state.canvas.shape_count(), 1);
    assert!(state.can_undo());
    assert!(!state.can_redo());

    state.undo();
    assert_eq!(state.canvas.shape_count(), 0);
    assert!(!state.can_undo());
    assert!(state.can_redo());

    state.redo();
    assert_eq!(state.canvas.shape_count(), 1);
    assert!(state.can_undo());
    assert!(!state.can_redo());
    assert!(
        state.canvas.get_shape(id).is_some(),
        "Shape should exist after redo"
    );
}

#[test]
fn undo_redo_multiple_shapes_sequence() {
    let mut state = DesignerState::new();

    let r = Rectangle::new(0.0, 0.0, 20.0, 10.0);
    let id1 = state.add_shape_with_undo(Shape::Rectangle(r));

    let c = Circle::new(Point::new(50.0, 50.0), 15.0);
    let id2 = state.add_shape_with_undo(Shape::Circle(c));

    let r2 = Rectangle::new(70.0, 70.0, 10.0, 10.0);
    let id3 = state.add_shape_with_undo(Shape::Rectangle(r2));

    assert_eq!(state.canvas.shape_count(), 3);

    // Undo last
    state.undo();
    assert_eq!(state.canvas.shape_count(), 2);
    assert!(state.canvas.get_shape(id3).is_none());

    // Undo second
    state.undo();
    assert_eq!(state.canvas.shape_count(), 1);
    assert!(state.canvas.get_shape(id2).is_none());
    assert!(state.canvas.get_shape(id1).is_some());

    // Redo both
    state.redo();
    assert_eq!(state.canvas.shape_count(), 2);
    state.redo();
    assert_eq!(state.canvas.shape_count(), 3);

    // All shapes present
    assert!(state.canvas.get_shape(id1).is_some());
    assert!(state.canvas.get_shape(id2).is_some());
    assert!(state.canvas.get_shape(id3).is_some());
}

#[test]
fn undo_then_new_action_clears_redo_stack() {
    let mut state = DesignerState::new();

    let r = Rectangle::new(0.0, 0.0, 10.0, 10.0);
    let _id1 = state.add_shape_with_undo(Shape::Rectangle(r));

    let c = Circle::new(Point::new(30.0, 30.0), 5.0);
    let _id2 = state.add_shape_with_undo(Shape::Circle(c));
    assert_eq!(state.canvas.shape_count(), 2);

    // Undo second
    state.undo();
    assert_eq!(state.canvas.shape_count(), 1);
    assert!(state.can_redo());

    // Add new shape — should clear redo stack
    let r2 = Rectangle::new(60.0, 60.0, 20.0, 20.0);
    let _id3 = state.add_shape_with_undo(Shape::Rectangle(r2));
    assert_eq!(state.canvas.shape_count(), 2);
    assert!(
        !state.can_redo(),
        "Redo stack should be cleared after new action"
    );
}

#[test]
fn undo_all_then_redo_all() {
    let mut state = DesignerState::new();

    let shapes: Vec<Shape> = (0..5)
        .map(|i| Shape::Rectangle(Rectangle::new(i as f64 * 20.0, 0.0, 10.0, 10.0)))
        .collect();

    let ids: Vec<u64> = shapes
        .into_iter()
        .map(|s| state.add_shape_with_undo(s))
        .collect();

    assert_eq!(state.canvas.shape_count(), 5);

    // Undo all
    for i in (0..5).rev() {
        state.undo();
        assert_eq!(state.canvas.shape_count(), i);
    }
    assert!(!state.can_undo());

    // Redo all
    for i in 1..=5 {
        state.redo();
        assert_eq!(state.canvas.shape_count(), i);
    }
    assert!(!state.can_redo());

    // Verify all shapes restored
    for id in &ids {
        assert!(
            state.canvas.get_shape(*id).is_some(),
            "Shape {id} should exist after full redo"
        );
    }
}

#[test]
fn state_consistency_after_gcode_generation() {
    let mut state = DesignerState::new();
    state.set_tool_diameter(3.175);
    state.set_feed_rate(500.0);
    state.set_spindle_speed(10000);

    let rect = Rectangle::new(10.0, 10.0, 40.0, 30.0);
    let id = state.add_shape_with_undo(Shape::Rectangle(rect));

    // Generate G-code
    let gcode1 = state.generate_gcode();
    assert!(!gcode1.is_empty());
    assert!(state.gcode_generated);

    // Undo should still work after G-code generation
    state.undo();
    assert_eq!(state.canvas.shape_count(), 0);

    // Redo
    state.redo();
    assert_eq!(state.canvas.shape_count(), 1);
    assert!(state.canvas.get_shape(id).is_some());

    // Generate again — should be identical
    let gcode2 = state.generate_gcode();
    assert_eq!(gcode1, gcode2, "G-code should be deterministic");
}

#[test]
fn clear_canvas_and_history_independence() {
    let mut state = DesignerState::new();

    let r = Rectangle::new(0.0, 0.0, 10.0, 10.0);
    let _id = state.add_shape_with_undo(Shape::Rectangle(r));
    assert_eq!(state.canvas.shape_count(), 1);

    // Clear canvas via clear_canvas
    state.clear_canvas();
    assert_eq!(state.canvas.shape_count(), 0);

    // clear_canvas pushes its own command, so undo should restore
    state.undo();
    assert_eq!(
        state.canvas.shape_count(),
        1,
        "Undo after clear_canvas should restore shapes"
    );
}

#[test]
fn undo_beyond_empty_is_noop() {
    let mut state = DesignerState::new();
    assert!(!state.can_undo());

    // Calling undo when nothing to undo should not panic
    state.undo();
    assert_eq!(state.canvas.shape_count(), 0);
    assert!(!state.can_undo());
    assert!(!state.can_redo());
}

#[test]
fn redo_beyond_empty_is_noop() {
    let mut state = DesignerState::new();

    let r = Rectangle::new(0.0, 0.0, 10.0, 10.0);
    let _id = state.add_shape_with_undo(Shape::Rectangle(r));

    // No redo available
    assert!(!state.can_redo());
    state.redo();
    assert_eq!(
        state.canvas.shape_count(),
        1,
        "Redo on empty redo stack should be noop"
    );
}

#[test]
fn mixed_shape_types_undo_redo() {
    let mut state = DesignerState::new();

    let r = Rectangle::new(0.0, 0.0, 20.0, 10.0);
    let _id1 = state.add_shape_with_undo(Shape::Rectangle(r));

    let c = Circle::new(Point::new(50.0, 50.0), 15.0);
    let _id2 = state.add_shape_with_undo(Shape::Circle(c));

    assert_eq!(state.canvas.shape_count(), 2);

    // Undo circle
    state.undo();
    assert_eq!(state.canvas.shape_count(), 1);

    // Redo circle
    state.redo();
    assert_eq!(state.canvas.shape_count(), 2);

    // Undo both
    state.undo();
    state.undo();
    assert_eq!(state.canvas.shape_count(), 0);

    // Redo just the first
    state.redo();
    assert_eq!(state.canvas.shape_count(), 1);
}

#[test]
fn gcode_generation_empty_canvas() {
    let mut state = DesignerState::new();
    state.set_tool_diameter(3.175);
    state.set_feed_rate(500.0);

    let _gcode = state.generate_gcode();
    // Empty canvas may still produce header/footer
    assert!(
        !state.gcode_generated,
        "Empty canvas should not flag gcode_generated"
    );
}

#[test]
fn concurrent_independent_designer_state_undo() {
    // Two DesignerState instances should have completely independent undo stacks
    let mut state_a = DesignerState::new();
    let mut state_b = DesignerState::new();

    let r = Rectangle::new(0.0, 0.0, 10.0, 10.0);
    let _id_a = state_a.add_shape_with_undo(Shape::Rectangle(r.clone()));

    let c = Circle::new(Point::new(30.0, 30.0), 10.0);
    let _id_b = state_b.add_shape_with_undo(Shape::Circle(c));

    assert_eq!(state_a.canvas.shape_count(), 1);
    assert_eq!(state_b.canvas.shape_count(), 1);

    // Undo A only
    state_a.undo();
    assert_eq!(state_a.canvas.shape_count(), 0);
    assert_eq!(
        state_b.canvas.shape_count(),
        1,
        "B should be unaffected by A's undo"
    );

    // Redo A
    state_a.redo();
    assert_eq!(state_a.canvas.shape_count(), 1);
    assert_eq!(state_b.canvas.shape_count(), 1);
}
