# UI Framework Knowledge Base

## Migration to GTK4
The project has migrated from Slint to GTK4 for the UI framework.
This file is maintained as per agent instructions, but primary UI development is now focused on GTK4 / gtk-rs.

## GTK4 Strategies
- **Layouts**: Use `Box`, `Grid`, and `Overlay` for flexible layouts.
- **State Management**: Use `Rc<RefCell<State>>` for shared state between widgets and callbacks.
- **Drawing**: Use `DrawingArea` with `Cairo` for custom 2D rendering (Visualizer, Designer).
- **Styling**: Use CSS classes (e.g., `visualizer-osd`, `heading`) for consistent styling.
- **Events**: Use `EventController` (Motion, Scroll, Key, Click, Drag) for input handling.
