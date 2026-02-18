// Integration tests for Designer operations: copy, paste, delete, group, ungroup

use gcodekit5_designer::DesignerState;

/// Helper: create a DesignerState with two rectangles already added via DesignerState commands
fn state_with_two_rects() -> DesignerState {
    let mut state = DesignerState::new();
    state.set_mode(1); // Rectangle mode
    state.add_shape_at(50.0, 50.0, false);
    state.add_shape_at(150.0, 50.0, false);
    assert_eq!(state.canvas.shape_count(), 2);
    state
}

// ── Delete Tests ──────────────────────────────────────────────────────

#[test]
fn test_delete_selected_removes_shape() {
    let mut state = state_with_two_rects();
    let ids: Vec<u64> = state.canvas.shapes().map(|s| s.id).collect();

    // Select the first shape
    state.canvas.select_shape(ids[0], false);
    assert_eq!(state.canvas.selected_count(), 1);

    state.delete_selected();
    assert_eq!(state.canvas.shape_count(), 1);
}

#[test]
fn test_delete_with_no_selection_is_noop() {
    let mut state = state_with_two_rects();
    state.canvas.deselect_all();
    state.delete_selected();
    assert_eq!(state.canvas.shape_count(), 2);
}

#[test]
fn test_delete_multiple_selected() {
    let mut state = state_with_two_rects();
    state.canvas.select_all();
    assert_eq!(state.canvas.selected_count(), 2);

    state.delete_selected();
    assert_eq!(state.canvas.shape_count(), 0);
}

#[test]
fn test_delete_is_undoable() {
    let mut state = state_with_two_rects();
    state.canvas.select_all();
    state.delete_selected();
    assert_eq!(state.canvas.shape_count(), 0);

    state.undo();
    assert_eq!(state.canvas.shape_count(), 2);
}

// ── Copy / Paste Tests ────────────────────────────────────────────────

#[test]
fn test_copy_paste_basic() {
    let mut state = state_with_two_rects();
    let ids: Vec<u64> = state.canvas.shapes().map(|s| s.id).collect();

    state.canvas.select_shape(ids[0], false);
    state.copy_selected();
    assert!(!state.clipboard.is_empty());

    state.paste_at_location(200.0, 200.0);
    assert_eq!(state.canvas.shape_count(), 3);
}

#[test]
fn test_copy_empty_selection() {
    let mut state = state_with_two_rects();
    state.canvas.deselect_all();
    state.copy_selected();
    assert!(state.clipboard.is_empty());
}

#[test]
fn test_paste_empty_clipboard_is_noop() {
    let mut state = state_with_two_rects();
    state.clipboard.clear();
    state.paste_at_location(100.0, 100.0);
    assert_eq!(state.canvas.shape_count(), 2);
}

#[test]
fn test_paste_assigns_new_ids() {
    let mut state = state_with_two_rects();
    let original_ids: Vec<u64> = state.canvas.shapes().map(|s| s.id).collect();

    state.canvas.select_shape(original_ids[0], false);
    state.copy_selected();
    state.paste_at_location(200.0, 200.0);

    let all_ids: Vec<u64> = state.canvas.shapes().map(|s| s.id).collect();
    // The pasted shape should have a unique id
    assert_eq!(all_ids.len(), 3);
    let pasted_id = all_ids.iter().find(|id| !original_ids.contains(id));
    assert!(pasted_id.is_some(), "Pasted shape should have a new id");
}

#[test]
fn test_paste_selects_pasted_shapes() {
    let mut state = state_with_two_rects();
    let ids: Vec<u64> = state.canvas.shapes().map(|s| s.id).collect();

    state.canvas.select_shape(ids[0], false);
    state.copy_selected();
    state.paste_at_location(200.0, 200.0);

    // Pasted shapes should be selected
    let selected: Vec<_> = state.canvas.shapes().filter(|s| s.selected).collect();
    assert_eq!(selected.len(), 1);
    // The selected shape should not be one of the originals
    assert!(!original_ids_contain(selected[0].id, &ids));
}

fn original_ids_contain(id: u64, ids: &[u64]) -> bool {
    ids.contains(&id)
}

#[test]
fn test_paste_multiple_shapes() {
    let mut state = state_with_two_rects();
    state.canvas.select_all();
    state.copy_selected();
    state.paste_at_location(300.0, 300.0);
    assert_eq!(state.canvas.shape_count(), 4);
}

#[test]
fn test_paste_is_undoable() {
    let mut state = state_with_two_rects();
    let ids: Vec<u64> = state.canvas.shapes().map(|s| s.id).collect();

    state.canvas.select_shape(ids[0], false);
    state.copy_selected();
    state.paste_at_location(200.0, 200.0);
    assert_eq!(state.canvas.shape_count(), 3);

    state.undo();
    assert_eq!(state.canvas.shape_count(), 2);
}

// ── Group / Ungroup Tests ─────────────────────────────────────────────

#[test]
fn test_group_selected() {
    let mut state = state_with_two_rects();
    state.canvas.select_all();
    assert!(state.can_group());

    state.group_selected();

    // Both shapes should now share a group_id
    let group_ids: Vec<Option<u64>> = state.canvas.shapes().map(|s| s.group_id).collect();
    assert!(group_ids.iter().all(|g| g.is_some()));
    assert_eq!(group_ids[0], group_ids[1]);
}

#[test]
fn test_group_requires_at_least_two() {
    let mut state = DesignerState::new();
    state.set_mode(1);
    state.add_shape_at(50.0, 50.0, false);
    let ids: Vec<u64> = state.canvas.shapes().map(|s| s.id).collect();
    state.canvas.select_shape(ids[0], false);

    assert!(!state.can_group());
    state.group_selected(); // should be a no-op
    let has_group = state.canvas.shapes().any(|s| s.group_id.is_some());
    assert!(!has_group);
}

#[test]
fn test_ungroup_selected() {
    let mut state = state_with_two_rects();
    state.canvas.select_all();
    state.group_selected();

    // Verify grouped
    assert!(state.canvas.shapes().all(|s| s.group_id.is_some()));

    // Now ungroup
    state.canvas.select_all();
    assert!(state.can_ungroup());
    state.ungroup_selected();

    // All group_ids should be None
    assert!(state.canvas.shapes().all(|s| s.group_id.is_none()));
}

#[test]
fn test_group_is_undoable() {
    let mut state = state_with_two_rects();
    state.canvas.select_all();
    state.group_selected();
    assert!(state.canvas.shapes().all(|s| s.group_id.is_some()));

    state.undo();
    assert!(state.canvas.shapes().all(|s| s.group_id.is_none()));
}

#[test]
fn test_ungroup_is_undoable() {
    let mut state = state_with_two_rects();
    state.canvas.select_all();
    state.group_selected();
    state.canvas.select_all();
    state.ungroup_selected();
    assert!(state.canvas.shapes().all(|s| s.group_id.is_none()));

    state.undo();
    assert!(state.canvas.shapes().all(|s| s.group_id.is_some()));
}

#[test]
fn test_cannot_ungroup_ungrouped_shapes() {
    let mut state = state_with_two_rects();
    state.canvas.select_all();
    assert!(!state.can_ungroup());
}

// ── Copy-Paste preserves group relationships ──────────────────────────

#[test]
fn test_paste_preserves_group_ids() {
    let mut state = state_with_two_rects();
    state.canvas.select_all();
    state.group_selected();

    // Copy the grouped shapes
    state.canvas.select_all();
    state.copy_selected();
    state.paste_at_location(300.0, 300.0);
    assert_eq!(state.canvas.shape_count(), 4);

    // Pasted shapes should be grouped together with a new group id
    let selected: Vec<_> = state.canvas.shapes().filter(|s| s.selected).collect();
    assert_eq!(selected.len(), 2);
    assert!(selected.iter().all(|s| s.group_id.is_some()));
    let pasted_group = selected[0].group_id;
    assert_eq!(selected[1].group_id, pasted_group);

    // The pasted group_id should differ from the original group_id
    let originals: Vec<_> = state.canvas.shapes().filter(|s| !s.selected).collect();
    assert_ne!(originals[0].group_id, pasted_group);
}

// ── Clear Canvas ──────────────────────────────────────────────────────

#[test]
fn test_clear_canvas() {
    let mut state = state_with_two_rects();
    state.clear_canvas();
    assert_eq!(state.canvas.shape_count(), 0);
}

#[test]
fn test_clear_canvas_is_undoable() {
    let mut state = state_with_two_rects();
    state.clear_canvas();
    assert_eq!(state.canvas.shape_count(), 0);
    state.undo();
    assert_eq!(state.canvas.shape_count(), 2);
}
