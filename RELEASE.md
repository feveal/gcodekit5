## [0.50.1-alpha.0] - 2026-01-19

### Changed
- **Code Quality**: Replaced 8 manual `impl Default` implementations with `#[derive(Default)]`
  - `MeasurementSystem` in `gcodekit5-core/src/units.rs`

  - `FeedRateUnits` in `gcodekit5-core/src/units.rs`
  - `DeviceType` in `gcodekit5-devicedb/src/model.rs`
  - `ControllerType` in `gcodekit5-devicedb/src/model.rs`
  - `Theme` in `gcodekit5-settings/src/config.rs`
  - `StartupTab` in `gcodekit5-settings/src/config.rs`
  - `OperationType` in `gcodekit5-designer/src/shapes.rs`
  - `PocketStrategy` in `gcodekit5-designer/src/pocket_operations.rs`
  - Eliminated 7 clippy `derivable_impls` warnings
  - Reduced code size and improved maintainability
  - REMEDIATION_PLAN.md Task 2.1.1 completed

- **Code Quality**: Fixed 18 field assignments outside initializers
  - Refactored `CuttingParameters` initialization in `materials.rs` (3 instances)
  - Refactored `CuttingParameters` initialization in `materials_mpi_static.rs` (9 instances)
  - Refactored `Stroke` initialization in `designer/renderer.rs` (4 instances)
  - Changed from `let mut obj = Default::default(); obj.field = value;` pattern to struct initialization with `..Default::default()`
  - Eliminated all 18 clippy `field_reassign_with_default` warnings
  - Improved code readability and Rust idiomaticity
  - REMEDIATION_PLAN.md Task 2.1.2 completed

- **Code Quality**: Replaced 15 manual clamp patterns with `.clamp()` method
  - `gcodekit5-core/src/data/mod.rs`: WCS clamping (1 instance)
  - `gcodekit5-settings/src/persistence.rs`: Grid line width clamping (2 instances)
  - `gcodekit5-designer/src/model.rs`: Line projection parameter clamping
  - `gcodekit5-designer/src/parametric.rs`: Constraint value clamping
  - `gcodekit5-designer/src/designer_state.rs`: Corner radius clamping
  - `gcodekit5-designer/src/viewport.rs`: Zoom level clamping
  - `gcodekit5-designer/src/multipass.rs`: Ramp progress clamping
  - `gcodekit5-ui/src/ui/themes.rs`: Font and spacing scale clamping (2 instances)
  - `gcodekit5-ui/src/ui/gtk/renderer_3d.rs`: Arc segment count clamping
  - `gcodekit5-ui/src/ui/gtk/designer_toolbox.rs`: Simulation resolution clamping
  - `gcodekit5-ui/src/ui/gtk/visualizer.rs`: Zoom and pane position clamping (4 instances)
  - Eliminated all 4 clippy `manual_clamp` warnings
  - Improved code clarity and reduced potential for errors
  - REMEDIATION_PLAN.md Task 2.1.3 completed

- **Code Quality**: Replaced 6 boilerplate trait implementations with derive macros
  - `FontKey` in `gcodekit5-designer/src/font_manager.rs`: Added `#[derive(PartialEq, Hash)]`, removed manual impls
  - `EventDispatcher` in `gcodekit5-core/src/core/event.rs`: Added `#[derive(Clone)]`, removed manual impl
  - `MessageDispatcher` in `gcodekit5-core/src/core/message.rs`: Added `#[derive(Clone)]`, removed manual impl
  - `NotificationManager` in `gcodekit5-ui/src/ui/notifications.rs`: Added `#[derive(Clone)]`, removed manual impl
  - `CommandNumberGenerator` in `gcodekit5-visualizer/src/gcode/mod.rs`: Added `#[derive(Clone)]`, removed manual impl
  - Removed unused `hash::{Hash, Hasher}` import from font_manager.rs
  - Reduced boilerplate code by ~40 lines
  - Improved maintainability and reduced chance of implementation errors
  - REMEDIATION_PLAN.md Task 2.1.4 completed

- **Code Quality**: Removed unnecessary `.clone()` calls on Copy types
  - `gcodekit5-camtools/src/gerber.rs`: Replaced `mode.clone()` with `*mode` for InterpolationMode
  - `gcodekit5-camtools/src/gerber.rs`: Replaced `mode.clone()` with `*mode` for QuadrantMode
  - Eliminated all 2 clippy `clone_on_copy` warnings
  - Improved code efficiency (avoids unnecessary clone overhead on Copy types)
  - REMEDIATION_PLAN.md Task 2.1.5 completed

- **Code Quality**: Collapsed 5 nested if blocks into simpler expressions
  - `gcodekit5-camtools/src/gerber.rs`: 2 else-if blocks for angle normalization
  - `gcodekit5-designer/src/pocket_operations.rs`: Combined edge crossing check with division guard
  - `gcodekit5-designer/src/selection_manager.rs`: Combined candidate and hit test checks
  - `gcodekit5-designer/src/designer_state.rs`: Combined selected and name-changed checks
  - Eliminated all 5 clippy `collapsible_if` and `collapsible_else_if` warnings
  - Improved code readability with flatter control flow
  - REMEDIATION_PLAN.md Task 2.1.6 completed

- **Code Quality**: Modularized cam_tools.rs (5,837 lines) into 11 focused modules
  - Split monolithic file into `crates/gcodekit5-ui/src/ui/gtk/cam_tools/` directory
  - Created modules:
    - `mod.rs` (507 lines) - CamToolsView with dashboard and re-exports
    - `common.rs` (58 lines) - Shared utilities (set_paned_initial_fraction, create_dimension_row)
    - `jigsaw.rs` (555 lines) - JigsawTool for puzzle generation
    - `bitmap_engraving.rs` (828 lines) - BitmapEngravingTool for raster image engraving
    - `vector_engraving.rs` (965 lines) - VectorEngravingTool for SVG/DXF engraving
    - `tabbed_box.rs` (783 lines) - TabbedBoxMaker for finger joint boxes
    - `speeds_feeds.rs` (196 lines) - SpeedsFeedsTool calculator
    - `spoilboard_surfacing.rs` (442 lines) - SpoilboardSurfacingTool
    - `spoilboard_grid.rs` (419 lines) - SpoilboardGridTool
    - `gerber.rs` (719 lines) - GerberTool for PCB milling
    - `drill_press.rs` (566 lines) - DrillPressTool for drilling operations
  - All modules under 1,000 lines (largest is 965 lines)
  - No circular dependencies, public API unchanged
  - Improved code organization and maintainability
  - REMEDIATION_PLAN.md Task 2.2.1 completed

## [0.50.0-alpha.0] - 2026-01-10

### Added
- **Designer**: Added aspect ratio lock checkbox to shape inspector
  - New "Lock Aspect" checkbox in the Size section of the Properties Panel
  - When enabled, changing width automatically adjusts height (and vice versa) to maintain aspect ratio
  - Aspect ratio is captured from current dimensions when lock is enabled
  - Works with both Enter/activate and focus-out events for width and height entries
