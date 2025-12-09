# UI Framework Knowledge Base

## Migration to GTK4
The project has migrated from Slint to GTK4 for the UI framework.
This file is maintained as per agent instructions, but primary UI development is now focused on GTK4 / gtk-rs.

-## Slint archival notes
 - The `.slint` UI files and legacy Slint Rust UI modules (under `crates/gcodekit5-ui/ui`) have been removed.
 - The Slint compatibility shim crate (previously `crates/slint`) and the Slint UI bridge (`editor_bridge.rs`) have been removed from the default workspace; the `slint_legacy_tests` feature remains to allow legacy tests if re-enabled in CI.
 - A non-UI editor bridge `EditorBridgeBackend` exists in `gcodekit5-gcodeeditor` to decouple core editor functionality from UI bridges.
 - The `src/app/callbacks` legacy callback modules were archived to `legacy/callbacks/` and the mainline `src/app/callbacks` directory was removed from the default workspace. These archived callbacks are kept for reference and restoration if needed.

## GTK4 Strategies
- **Layouts**: Use `Box`, `Grid`, and `Overlay` for flexible layouts.
- **State Management**: Use `Rc<RefCell<State>>` for shared state between widgets and callbacks.
- **Drawing**: Use `DrawingArea` with `Cairo` for custom 2D rendering (Visualizer, Designer).
- **Styling**: Use CSS classes (e.g., `visualizer-osd`, `heading`) for consistent styling.
- **Events**: Use `EventController` (Motion, Scroll, Key, Click, Drag) for input handling.

## Visualizer Implementation (GTK4)
- **Performance**: Use `RenderCache` to store expensive calculations (e.g., intensity buckets) and rebuild only when G-code changes.
- **LOD**: Implement Level of Detail (LOD) rendering based on zoom level to maintain performance with large files.
- **Interaction**: Use `GestureDrag` for panning and `EventControllerScroll` for zooming.
- **Overlays**: Use `Overlay` widget to place floating controls (buttons, status labels) on top of the `DrawingArea`.
- **Coordinate Systems**: Use a unified `Viewport` struct to manage World-to-Screen transformations. Ensure consistent mapping between mouse events (screen coordinates) and canvas objects (world coordinates).

## Designer Migration
- The Designer component has been fully migrated to GTK4, including all tools (Select, Pan, Shapes), property editing, and CAM operations.
- The migration plan in `DESIGNER_MIGRATION_COMPARISON.md` is now largely complete.
