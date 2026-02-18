use gcodekit5_gcodeeditor::EditorState;

#[test]
fn test_new_editor_is_empty() {
    let editor = EditorState::new(400.0, 20.0);
    assert_eq!(editor.get_text(), "");
    assert_eq!(editor.cursor_pos(), 0);
    assert!(!editor.is_modified());
    assert_eq!(editor.line_count(), 1);
    assert_eq!(editor.char_count(), 0);
}

#[test]
fn test_load_text() {
    let mut editor = EditorState::new(400.0, 20.0);
    editor.load_text("G0 X10 Y20\nG1 Z-5 F100\n");
    assert_eq!(editor.line_count(), 3);
    assert_eq!(editor.cursor_pos(), 0);
    assert!(!editor.is_modified());
}

#[test]
fn test_load_text_clears_undo() {
    let mut editor = EditorState::new(400.0, 20.0);
    editor.insert_text("Hello");
    assert!(editor.can_undo());
    editor.load_text("New text");
    assert!(!editor.can_undo());
    assert!(!editor.is_modified());
}

#[test]
fn test_cursor_movement() {
    let mut editor = EditorState::new(400.0, 20.0);
    editor.load_text("Hello World");
    editor.set_cursor(5);
    assert_eq!(editor.cursor_pos(), 5);

    let (line, col) = editor.cursor_line_col();
    assert_eq!(line, 0);
    assert_eq!(col, 5);
}

#[test]
fn test_cursor_clamp() {
    let mut editor = EditorState::new(400.0, 20.0);
    editor.load_text("Hi");
    editor.set_cursor(100);
    assert_eq!(editor.cursor_pos(), 2);
}

#[test]
fn test_cursor_multiline() {
    let mut editor = EditorState::new(400.0, 20.0);
    editor.load_text("Line 1\nLine 2\nLine 3");

    let pos = editor.line_col_to_char(1, 3);
    editor.set_cursor(pos);
    let (line, col) = editor.cursor_line_col();
    assert_eq!(line, 1);
    assert_eq!(col, 3);
}

#[test]
fn test_mark_unmodified() {
    let mut editor = EditorState::new(400.0, 20.0);
    editor.insert_text("changes");
    assert!(editor.is_modified());
    editor.mark_unmodified();
    assert!(!editor.is_modified());
}

#[test]
fn test_insert_at_cursor() {
    let mut editor = EditorState::new(400.0, 20.0);
    editor.insert_text("Hello");
    editor.insert_text(" World");
    assert_eq!(editor.get_text(), "Hello World");
    assert_eq!(editor.cursor_pos(), 11);
}

#[test]
fn test_insert_newlines() {
    let mut editor = EditorState::new(400.0, 20.0);
    editor.insert_text("Line1\nLine2\nLine3");
    assert_eq!(editor.line_count(), 3);
    assert_eq!(editor.get_line(0), Some("Line1\n".to_string()));
    assert_eq!(editor.get_line(2), Some("Line3".to_string()));
}

#[test]
fn test_delete_forward() {
    let mut editor = EditorState::new(400.0, 20.0);
    editor.load_text("Hello");
    editor.set_cursor(0);
    editor.delete_forward(2);
    assert_eq!(editor.get_text(), "llo");
}

#[test]
fn test_delete_forward_at_end() {
    let mut editor = EditorState::new(400.0, 20.0);
    editor.load_text("Hi");
    editor.set_cursor(2);
    editor.delete_forward(1);
    assert_eq!(editor.get_text(), "Hi");
}

#[test]
fn test_delete_backward_at_start() {
    let mut editor = EditorState::new(400.0, 20.0);
    editor.load_text("Hi");
    editor.set_cursor(0);
    editor.delete_backward(1);
    assert_eq!(editor.get_text(), "Hi");
}

#[test]
fn test_undo_returns_false_when_empty() {
    let mut editor = EditorState::new(400.0, 20.0);
    assert!(!editor.undo());
}

#[test]
fn test_redo_returns_false_when_empty() {
    let mut editor = EditorState::new(400.0, 20.0);
    assert!(!editor.redo());
}

#[test]
fn test_multiple_undo_redo() {
    let mut editor = EditorState::new(400.0, 20.0);
    editor.insert_text("A");
    editor.insert_text("B");
    editor.insert_text("C");
    assert_eq!(editor.get_text(), "ABC");

    editor.undo();
    assert_eq!(editor.get_text(), "AB");
    editor.undo();
    assert_eq!(editor.get_text(), "A");
    editor.undo();
    assert_eq!(editor.get_text(), "");

    editor.redo();
    assert_eq!(editor.get_text(), "A");
    editor.redo();
    assert_eq!(editor.get_text(), "AB");
}

#[test]
fn test_get_line() {
    let mut editor = EditorState::new(400.0, 20.0);
    editor.load_text("G0 X10\nG1 Y20\nG2 Z30");
    assert_eq!(editor.get_line(0), Some("G0 X10\n".to_string()));
    assert_eq!(editor.get_line(1), Some("G1 Y20\n".to_string()));
    assert_eq!(editor.get_line(2), Some("G2 Z30".to_string()));
    assert_eq!(editor.get_line(99), None);
}

#[test]
fn test_get_visible_lines_empty() {
    let editor = EditorState::new(400.0, 20.0);
    let (start, lines) = editor.get_visible_lines();
    assert_eq!(start, 0);
    assert!(!lines.is_empty());
}

#[test]
fn test_scroll_operations() {
    let mut editor = EditorState::new(400.0, 20.0);
    let text: String = (0..100).map(|i| format!("Line {}\n", i)).collect();
    editor.load_text(&text);

    editor.scroll_by(10);
    let vp = editor.viewport();
    assert_eq!(vp.start_line, 10);

    editor.scroll_to_line(50);
    let vp = editor.viewport();
    assert_eq!(vp.scroll_offset, 50);
}

#[test]
fn test_set_viewport_size() {
    let mut editor = EditorState::new(400.0, 20.0);
    editor.set_viewport_size(800.0, 16.0);
    assert_eq!(editor.viewport().visible_lines, 50);
}

#[test]
fn test_unicode_text() {
    let mut editor = EditorState::new(400.0, 20.0);
    editor.insert_text("héllo wörld 日本語");
    assert_eq!(editor.get_text(), "héllo wörld 日本語");
    assert!(editor.char_count() > 0);
    assert!(editor.is_modified());
}

#[test]
fn test_large_text_load() {
    let mut editor = EditorState::new(400.0, 20.0);
    let text: String = (0..10000)
        .map(|i| format!("G1 X{} Y{} F1000\n", i, i * 2))
        .collect();
    editor.load_text(&text);
    assert_eq!(editor.line_count(), 10001);
    assert!(!editor.is_modified());
}
