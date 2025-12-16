# Plan: Clean Slate Designer Backend with `csgrs`

This plan outlines a complete re-architecture of the `gcodekit5-designer` backend. Instead of wrapping a generic sketch, we will implement a **Type-Specific System** where each shape preserves its identity and logic (Parametric) until a boolean operation forces it to become generic (Static).

## Phase 1: Core Architecture & Traits (Completed)

1.  **Define `DesignerShape` Trait**:
    *   Create `crates/gcodekit5-designer/src/model.rs`.
    *   Trait methods:
        *   `render(&self) -> lyon::path::Path`: For UI display.
        *   `as_csg(&self) -> csgrs::CSG`: For boolean operations.
        *   `bounds(&self) -> Rect`: For selection/layout.
        *   `transform(&mut self, t: Transform)`: For moving/scaling.
        *   `properties(&self) -> Vec<Property>`: For the UI property editor.

2.  **Define the Shape Enum**:
    *   The main storage type will be an enum holding the specific implementations:
    ```rust
    pub enum Shape {
        Rectangle(DesignRectangle),
        Circle(DesignCircle),
        Path(DesignPath), // The generic fallback
        // Add others as needed (Text, Polygon, etc.)
    }
    ```

## Phase 2: Native Primitive Implementations (Completed)

3.  **Implement `DesignRectangle`**:
    *   **Backing**: `csgrs::primitives::Rectangle` (or equivalent).
    *   **Logic**: Maintains `width`, `height`, `center`, `corner_radius`.
    *   **Behavior**:
        *   `render()`: Generates path from parameters.
        *   `as_csg()`: Returns the `csgrs` primitive.
        *   `properties()`: Exposes "Width", "Height", "Corner Radius".

4.  **Implement `DesignCircle`**:
    *   **Backing**: `csgrs::primitives::Circle`.
    *   **Logic**: Maintains `radius`, `center`.
    *   **Behavior**:
        *   `render()`: Generates circle path.
        *   `as_csg()`: Returns the `csgrs` primitive.
        *   `properties()`: Exposes "Radius".

5.  **Implement `DesignPath` (The Generic Fallback)**:
    *   **Backing**: `csgrs::Polygon` or `csgrs::Sketch`.
    *   **Logic**: Holds raw vertex/curve data.
    *   **Behavior**:
        *   `render()`: Converts polygon/sketch to `lyon` path.
        *   `as_csg()`: Returns self as CSG.
        *   `properties()`: Exposes generic info (e.g., "Vertices", "Area").

## Phase 3: Boolean Operations (The Transition) (Completed)

6.  **Implement Boolean Logic**:
    *   Create `ops::perform_boolean(a: &Shape, b: &Shape, op: BooleanOp) -> Shape`.
    *   **Process**:
        1.  Call `a.as_csg()` and `b.as_csg()`.
        2.  Perform `csgrs::union/diff/intersect`.
        3.  **Crucial Step**: The result is a `csgrs::CSG` (complex).
        4.  Wrap the result in `Shape::Path(DesignPath::from_csg(...))`.
    *   *Outcome*: Parametric shapes (`Rect`, `Circle`) become `Path` shapes after interaction. This is standard vector editor behavior.

## Phase 4: Integration & Migration (Completed)

7.  [x] **Update `Canvas`**:
    *   Replace the old `Shape` list with `Vec<Shape>`.
    *   Update selection logic to use the `DesignerShape` trait methods.

8.  [x] **Update UI/Property Editor**:
    *   The Property Editor now queries `shape.properties()`.
    *   If `Shape::Rectangle`, it shows width/height inputs.
    *   If `Shape::Path`, it shows generic info.
    *   **Added Boolean Operation Buttons**: Added Union, Difference, and Intersection buttons to the Designer Toolbox.
    *   **Selection Logic**: Boolean buttons are only enabled when 2 or more shapes are selected.
    *   **UI Fix**: Updated boolean operation handlers to refresh the properties panel immediately after the operation, ensuring the new shape's properties are shown without requiring re-selection.

9.  [x] **Fix Build Errors**:
    *   Resolve all compilation errors in `gcodekit5-designer` and `gcodekit5-ui`.

## Phase 5: Verification (Completed)

9.  [x] **Test Parametric Editing**:
    *   Create Rect -> Change Width -> Verify Render updates.
10. [x] **Test Boolean Transition**:
    *   Create Rect + Circle -> Union -> Verify result is `Shape::Path`.
    *   Verify the new Path looks correct.
11. [x] **Check SVG and DXF Import**:
    *   Verified SVG import uses `csgrs` and improved robustness.
    *   Verified DXF import works with basic entities.

