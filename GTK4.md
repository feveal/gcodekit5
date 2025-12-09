# GTK4 & Rust Insights

## Viewport & Canvas Sizing
- **Issue**: When using a `DrawingArea` with a backend `Viewport` or `Canvas` struct that manages zoom/pan, the backend needs to be explicitly updated with the widget's dimensions.
- **Solution**: Use `widget.set_draw_func` to update the backend size on every draw (which happens on resize), or use `connect_resize` / `connect_map`.
- **Gotcha**: `DrawingArea` dimensions might be 0 or default during early initialization. Use `connect_map` with a small timeout or check dimensions before applying "Fit to View" logic to ensure correct aspect ratio and padding.

## Coordinate Systems
- **Designer**: Uses Cartesian coordinates (Y-up).
- **GTK/Cairo**: Uses Screen coordinates (Y-down).
- **Transformation**: Always handle Y-flip in the `Viewport` logic.

## SourceView5 Search Implementation
- **SearchContext**: The `SearchContext` is central to search operations. It runs asynchronously.
- **Counting Matches**: To implement "n of m" (current match index of total matches), you must iterate through occurrences. `context.forward()` and `context.backward()` are useful but require careful handling of iterators.
- **Iterating**: To find the "current" match index relative to the cursor:
    1. Get the start of the buffer.
    2. Loop using `context.forward(iter)` until the match start is >= the cursor position (or the current match).
    3. Count the iterations.
- **Button State**: Use the count and total matches (from `context.occurrences_count()`) to enable/disable "Next"/"Previous" buttons.
- **API Gotchas**:
    - `buffer.get_selection_bound()` in C is `buffer.selection_bound()` in the Rust bindings.
    - `buffer.get_insert()` in C is `buffer.insert()` in the Rust bindings.
    - `search_settings.set_search_text(Some("text"))` is required; passing `None` clears it.
