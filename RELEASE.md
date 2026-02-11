## [0.51.0-alpha.0] - 2026-02-11

### Added
- **Shared error dialog helper** (`file_dialog::show_error_dialog`): modal, transient, with parent window support
- **Firmware settings file I/O** (issue #13): JSON-based load/save for `DefaultFirmwareSettings`
- **Visualizer dirty tracking** (issue #18): skip buffer regeneration when data unchanged
- **Comprehensive error types** for camtools, devicedb, gcodeeditor, settings crates (thiserror)
- **Null/invalid state guards**: debug_assert checks, NaN/Inf guards, state transition validation

### Changed
- **Task 3.1 — Reduce Complex Type Nesting**: Adopted `gcodekit5_core::types::aliases` (`Shared<T>`, `ThreadSafe<T>`, `ThreadSafeRw<T>`, etc.) across entire workspace, replacing ~180 raw `Rc<RefCell<T>>`, `Arc<Mutex<T>>`, and `Arc<RwLock<T>>` patterns with readable type aliases and constructor helpers (`shared()`, `thread_safe()`, `thread_safe_rw()`)
- **G10 Z-axis suppression**: G10 offset commands (L2/L20) now omit Z0 when device num_axes < 3 in vector_engraver, laser_engraver, jigsaw_puzzle, gerber, tabbed_box, and spoilboard_grid
- **Issue #15 — Editor error dialogs**: file read/save errors now show GTK4 MessageDialog with file path
- **Consolidated 5 duplicate `show_error_dialog` implementations** into shared helper
- **FileChooserNative → FileChooserDialog** across all file operations (KDE/Kubuntu compatibility)
- **Split 13 large files** into focused modules (largest file: 3,907 → 2,898 lines)
- **Clippy warnings**: reduced from 155 to 0 across entire workspace

### Fixed
- **KDE/Kubuntu file dialog crash**: replaced FileChooserNative with FileChooserDialog
- **Hard error** in pocket_operations.rs: `while_immutable_condition` (while true → loop)

### Removed
- All `println!`/`eprintln!` from test code (replaced with proper assertions)
- Unsafe `unwrap()`/`expect()` calls on runtime values across 14 files
