# Plan: Implement Hybrid 2D/3D Visualizer

This plan details the tasks to extend the existing 2D G-code visualizer to support 3D visualization using OpenGL, while retaining the 2D Cairo-based renderer for laser jobs and other 2D-specific use cases. The UI will allow switching between modes.

## Prerequisites
- Ensure `gtk4` features for OpenGL are enabled.
- Familiarity with OpenGL concepts (shaders, VBOs, VAOs, matrices).

## Steps

### Step 1: Update Data Structures for 3D Compatibility
**Goal:** Extend core data structures to support Z-axis while maintaining 2D compatibility.
- **File:** `crates/gcodekit5-visualizer/src/visualizer/visualizer_2d.rs` (rename to `visualizer.rs`)
- **Tasks:**
    - Rename `Point2D` to `Point3D` and add `z: f32`.
    - Update `GCodeCommand` variants (`Move`, `Arc`, `Dwell`) to use `Point3D`.
    - Update `Bounds` struct to track `min_z` and `max_z`.
    - Update `Visualizer2D` struct to `Visualizer` and include Z-axis bounds.
    - **Note:** The 2D renderer will simply ignore the Z component or project it to 0.
- **Prompt:** Rename `visualizer_2d.rs` to `visualizer.rs`. Update `Point2D` to `Point3D` with a Z field, and update `GCodeCommand`, `Bounds`, and `Visualizer2D` (renamed to `Visualizer`) to support 3D coordinates. Ensure existing 2D logic compiles by using 0.0 for Z where needed.

### Step 2: Update G-Code Parser
**Goal:** Extract Z-coordinates from G-code.
- **File:** `crates/gcodekit5-visualizer/src/visualizer/visualizer.rs`
- **Tasks:**
    - Update `parse_linear_move` to look for 'Z' parameter.
    - Update `parse_arc_move` to handle helical arcs (Z movement during arc) or end-point Z.
    - Ensure current Z position is maintained in the parser state.
- **Prompt:** Update the G-code parser in `visualizer.rs` to extract Z-axis values. Modify `parse_linear_move` and `parse_arc_move` to capture Z coordinates and update the current position state.

### Step 3: Add 3D Math and OpenGL Dependencies
**Goal:** Add necessary crates for 3D math and OpenGL bindings.
- **File:** `crates/gcodekit5-visualizer/Cargo.toml` and `crates/gcodekit5-ui/Cargo.toml`
- **Tasks:**
    - Add `glam` or `nalgebra` for matrix/vector math.
    - Add `glow` for safe OpenGL bindings compatible with GTK4.
    - Run `cargo build` to ensure dependencies resolve.
- **Prompt:** Add `glam` and `glow` dependencies to `crates/gcodekit5-visualizer/Cargo.toml` and `crates/gcodekit5-ui/Cargo.toml`. Run `cargo build` to verify.

### Step 4: Implement 3D Camera and Viewport Logic
**Goal:** Manage 3D view state independent of 2D viewport.
- **File:** `crates/gcodekit5-visualizer/src/visualizer/camera.rs` (create new)
- **Tasks:**
    - Create a `Camera` struct with `eye`, `target`, and `up` vectors.
    - Implement methods for `orbit`, `pan`, and `zoom`.
    - Implement `get_view_matrix` and `get_projection_matrix`.
- **Prompt:** Create `crates/gcodekit5-visualizer/src/visualizer/camera.rs`. Implement a `Camera` struct with orbit, pan, and zoom capabilities, and methods to generate view and projection matrices using `glam`.

### Step 5: Create OpenGL Shader Management
**Goal:** Compile and link vertex and fragment shaders for 3D mode.
- **File:** `crates/gcodekit5-ui/src/ui/gtk/shaders.rs` (create new)
- **Tasks:**
    - Write vertex shader (MVP transform, color/intensity).
    - Write fragment shader (output color).
    - Implement `ShaderProgram` struct using `glow`.
- **Prompt:** Create `crates/gcodekit5-ui/src/ui/gtk/shaders.rs`. Implement a `ShaderProgram` struct that compiles vertex and fragment shaders and links them using `glow`. Include basic shaders for 3D rendering.

### Step 6: Implement Vertex Buffer Management
**Goal:** Convert G-code commands to GPU-friendly vertex data.
- **File:** `crates/gcodekit5-ui/src/ui/gtk/renderer_3d.rs` (create new)
- **Tasks:**
    - Create `RenderBuffers` struct for VBO/VAO IDs.
    - Implement generation of vertex data from `Visualizer` commands.
    - Batch rapid moves, cutting moves, and discretized arcs.
- **Prompt:** Create `crates/gcodekit5-ui/src/ui/gtk/renderer_3d.rs`. Implement `RenderBuffers` to manage VBOs/VAOs and a function to generate vertex data from `Visualizer` commands, batching moves for OpenGL rendering.

### Step 7: Implement 3D Grid and Axis Rendering
**Goal:** Render reference grid and axes in 3D mode.
- **File:** `crates/gcodekit5-ui/src/ui/gtk/renderer_3d.rs`
- **Tasks:**
    - Generate vertex data for XY plane grid (Z=0).
    - Generate vertex data for X (Red), Y (Green), Z (Blue) axes.
- **Prompt:** Extend `crates/gcodekit5-ui/src/ui/gtk/renderer_3d.rs` to include functions for generating and rendering a 3D grid on the XY plane and XYZ coordinate axes.

### Step 8: Implement View Switching UI
**Goal:** Create a container to switch between 2D and 3D views.
- **File:** `crates/gcodekit5-ui/src/ui/gtk/visualizer.rs`
- **Tasks:**
    - Create a `gtk::Stack` to hold the existing `DrawingArea` (2D) and a new `gtk::GLArea` (3D).
    - Add a `StackSwitcher` or a Toggle Button to the toolbar/sidebar to switch between "2D" and "3D" pages.
    - Ensure the `Visualizer` data is shared or accessible by both renderers.
- **Prompt:** Modify `crates/gcodekit5-ui/src/ui/gtk/visualizer.rs` to use a `gtk::Stack`. Add the existing `DrawingArea` as the "2D" page and a new `gtk::GLArea` as the "3D" page. Add controls to switch between them.

### Step 9: Implement 3D Input Handling & Sync
**Goal:** Handle input for 3D mode and sync state if needed.
- **File:** `crates/gcodekit5-ui/src/ui/gtk/visualizer.rs`
- **Tasks:**
    - Implement `GLArea` render callback using the 3D renderer.
    - Add input controllers (`GestureDrag`, `EventControllerScroll`) specifically for the `GLArea` to control the 3D camera.
    - Ensure 2D input handlers remain active only when 2D view is visible.
- **Prompt:** In `crates/gcodekit5-ui/src/ui/gtk/visualizer.rs`, implement the `render` signal for `GLArea` using `renderer_3d`. Add mouse event controllers to the `GLArea` for camera orbit, pan, and zoom.

### Step 10: Integration and Cleanup
**Goal:** Finalize hybrid visualizer.
- **File:** `crates/gcodekit5-ui/src/ui/gtk/visualizer.rs`
- **Tasks:**
    - Update `canvas_renderer.rs` (2D) to work with `Point3D` (ignoring Z).
    - Implement logic to auto-select 3D mode if Z-travel is detected (optional, or just default to 2D).
    - Verify "Fit to View" works correctly in both modes (2D uses 2D bounds, 3D uses 3D bounds/camera).
    - Test switching between modes during runtime.
- **Prompt:** Update `canvas_renderer.rs` to handle `Point3D` by ignoring Z. Ensure "Fit to View" works for both 2D and 3D modes in `visualizer.rs`. Verify mode switching works smoothly.

### Step 11: Implement Real-Time Tool Position Tracking
**Goal:** Visualize the laser/spindle position in real-time during G-code streaming.
- **File:** `crates/gcodekit5-ui/src/ui/gtk/visualizer.rs` and `crates/gcodekit5-ui/src/ui/gtk/renderer_3d.rs`
- **Tasks:**
    - Update `GcodeVisualizer` to accept Z coordinates in `set_current_position`.
    - In `renderer_3d.rs`, implement a `render_tool_marker` function to draw a 3D object (e.g., cone, cylinder, or crosshair) at the current tool coordinates.
    - Ensure the tool marker is drawn last (on top) or with proper depth testing.
    - Connect the machine status updates (from `DeviceManager` or event bus) to trigger `set_current_position` and `queue_render` for the active view (2D or 3D).
    - Optimize the render loop to handle frequent position updates without full geometry rebuilds (pass tool position as a uniform or update a small VBO).
- **Prompt:** Update `GcodeVisualizer` to track Z position. Implement `render_tool_marker` in `renderer_3d.rs` to draw a 3D tool indicator. Ensure `set_current_position` updates the 3D view efficiently.
