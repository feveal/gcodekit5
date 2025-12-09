Version: 0.2.5-alpha.5

## [0.2.5-alpha.5] - 2025-12-09

### Fixed
- **Visualizer**: Fixed "Fit to Device" functionality to correctly center the device working area in the view.
- **Designer**: Aligned "Fit to Device" logic with Visualizer to use consistent padding (5%) and default bounds.
- **Designer**: Fixed Pan tool jumping on drag start.
- **Designer**: Fixed padding discrepancy between Designer and Visualizer by syncing widget size to backend viewport.
- **Designer**: Fixed Origin markers to extend to world extent (Red/Green).

### Added
- **GCode Editor**: Implemented G-Code syntax highlighting with a custom "Bright" color scheme.
- **GCode Editor**: Added a floating Search/Replace panel with "n of m" match counting and navigation controls.
- **GCode Editor**: Implemented logic to enable Replace/Replace All buttons only when search matches exist and replacement text is provided.
- **GCode Editor**: Moved "Replace All" operation to a background thread to prevent UI freezing, with progress updates in the status bar.
- **Designer**: Added Device Bounds rendering (Blue, 2px wide) to visualize the working area.
- **Designer/Visualizer**: Ensured both views initialize to "Fit to Device" on startup.

### Changed
- **Refactor**: Major refactor of Designer and Visualizer integration to improve coordinate handling and rendering performance.
- **Designer**: Updated `DesignerCanvas` and `Viewport` to use improved coordinate transformation logic.
- **UI**: Updated GTK UI components (`designer.rs`, `visualizer.rs`, `main_window.rs`) to align with the new backend architecture.
- **Visualizer**: Enhanced 2D visualizer logic and coordinate transforms in `crates/gcodekit5-visualizer`.
- **Cleanup**: Removed unused helper functions in `src/app/helpers.rs` and refactored `src/app/mod.rs`.
