# Visualiser + Designer Tabs — Aesthetics & Usability Recommendations

Each item below is **separately implementable**, can be done **out of sequence**, and is **complete on its own**.
All recommendations assume the primary target display is **FHD @ 125% scaling**.

1. **Unify floating OSD styling and placement between Visualiser and Designer**
   - Change: Standardize both tabs on the same OSD component (CSS class, padding, corner radius, shadow, margins) and default placement (status bottom-left, view controls bottom-right).
   - Why: Consistent visual language reduces cognitive load when switching tabs.
   - Done when: Both tabs’ OSD panels look/feel identical and occupy similar screen real-estate.
   - Execution prompt: In the **Visualiser tab (canvas overlay/OSD)** and **Designer tab (canvas overlay/OSD)**, standardize the overlay containers to use the same CSS class names and margins, and place the status OSD at bottom-left and the view controls OSD at bottom-right.

2. **Replace OSD text buttons (“-”, “Fit”, “+”) with icon+label buttons (or icon-only with tooltips), keeping translation keys stable**
   - Change: Use `Button::from_icon_name(...)` + separate label widget (or icon-only) and set tooltip + accessible label via `AccessibleProperty::Label`.
   - Why: Text-only micro-controls are ambiguous and inconsistent with the rest of the UI.
   - Done when: Zoom/Fit controls are clear, theme-consistent, and accessible on both tabs.
   - Execution prompt: In the **Visualiser tab (canvas overlay/OSD: zoom/fit control cluster)** and **Designer tab (canvas overlay/OSD: zoom/fit control cluster)**, update each control to display an icon plus a separate label widget (do not embed icons in translated strings) and add tooltips + accessible labels.

3. **Add a consistent “Reset View” action to both tabs’ OSD controls**
   - Change: Provide a “Home/Reset view” button next to Fit/Zoom (same icon and tooltip on both tabs).
   - Why: Users need a fast escape hatch after pan/zoom disorientation.
   - Done when: One click restores a predictable default view in both tabs.
   - Execution prompt: In the **Visualiser tab (canvas overlay/OSD: view controls)** and **Designer tab (canvas overlay/OSD: view controls)**, add a new “Reset View” button (home/target icon) that returns the camera/pan/zoom to a deterministic default.

4. **Make OSD strings fully localizable and consistent**
   - Change: Wrap all visible strings and tooltips in `t!()` in both tabs (e.g., “Zoom In”, “Fit to View”, “2D View”, “3D View”, “Ready”, “Show Grid”).
   - Why: These tabs currently contain raw English strings.
   - Done when: `scripts/update-po.sh` extracts these strings and translators can cover them.
   - Execution prompt: In the **Visualiser tab (OSD + sidebar controls)** and **Designer tab (OSD + tool panels/status)**, wrap all user-visible strings and tooltips in `t!()` and ensure no `t!(format!(...))` patterns are introduced.

5. **Standardize coordinate readouts: show Zoom + Center + Cursor position in a single, consistent format**
   - Change: Define a shared format such as: `Zoom: 100%  Center: X… Y…  Cursor: X… Y… (mm/in)`.
   - Why: Visualiser currently shows center; Designer has a separate status bar and a different OSD string.
   - Done when: Both tabs expose the same baseline spatial feedback in the same place.
   - Execution prompt: In the **Visualiser tab (status OSD)** and **Designer tab (status OSD or status bar, whichever is primary)**, implement a shared status string formatter for zoom/center/cursor values and display it consistently in the same location.

6. **Add a shared “Units badge” in the status OSD (mm/in) and ensure both tabs respect MeasurementSystem**
   - Change: Display current unit label (from settings) and ensure any coordinate formatting uses `format_length`.
   - Why: Visual tools are easy to misread if unit context is hidden.
   - Done when: Screenshots of either tab clearly indicate displayed units.
   - Execution prompt: In the **Visualiser tab (status OSD)** and **Designer tab (status OSD/status bar)**, add a small units indicator (mm/in) bound to settings and ensure coordinate values are formatted via shared unit helpers (e.g., `format_length`).

7. **Normalize scroll/pan interaction model between Visualiser and Designer**
   - Change: Align gesture rules (scroll=pan vs scroll=zoom with Ctrl, drag=pan with middle mouse/Space, etc.) and document via tooltips/help.
   - Why: Inconsistent navigation is a major usability tax.
   - Done when: A user can pan/zoom in either tab with the same muscle memory.
   - Execution prompt: In the **Visualiser tab (canvas interaction handlers)** and **Designer tab (canvas interaction handlers)**, standardize mouse wheel, Ctrl+wheel, drag, and modifier key behavior to match one agreed navigation scheme.

8. **Make scrollbars optional and reduce always-on chrome**
   - Change: Provide a toggle (or auto-hide) for scrollbars in both tabs; default to hidden/overlay to save vertical/horizontal space.
   - Why: FHD @ 125% is tight; permanent scrollbars steal valuable pixels.
   - Done when: Users can reclaim canvas space while retaining navigation capability.
   - Execution prompt: In the **Visualiser tab (canvas container)** and **Designer tab (canvas container)**, switch scrollbars to overlay/auto-hide behavior (or add a per-tab toggle) so the default maximizes canvas space.

9. **Replace Visualiser’s tick-based `Paned` sizing with stable initial sizing (and remember user choice)**
   - Change: Remove the continuous `add_tick_callback` position forcing; set initial divider once (after realize/size-allocate), then persist user-chosen position.
   - Why: Tick resizing can feel “broken” and can fight user adjustments.
   - Done when: Divider stops snapping back; layout remains stable across sessions.
   - Execution prompt: In the **Visualiser tab (main paned split: sidebar ↔ canvas)**, remove any tick-based divider forcing; set a one-time initial position and persist/reload the user’s divider position.

10. **Unify “Fit to Device Working Area” control across both tabs**
   - Change: Use the same icon, tooltip, and placement for “Fit to Device” in Visualiser and Designer.
   - Why: It’s conceptually the same action and should be discoverable in the same location.
   - Done when: Both tabs expose the same affordance with consistent labeling.
   - Execution prompt: In the **Visualiser tab (view controls OSD)** and **Designer tab (view controls OSD)**, ensure there is a “Fit to Device Working Area” button with the same icon/tooltip and in the same relative position in the control cluster.

11. **Improve Visualiser view mode switching (2D/3D) to a compact segmented control**
   - Change: Replace or restyle the `StackSwitcher` to a compact toggle/segmented switch with icons + labels.
   - Why: Current switching can consume header space and doesn’t visually match other tab controls.
   - Done when: 2D/3D switching is compact, consistent, and obvious.
   - Execution prompt: In the **Visualiser tab (top controls/header area)**, replace the current 2D/3D switch UI with a compact segmented control using icons + labels, consistent with other button styling.

12. **Add a consistent “Legend” panel (colors + semantics) in both tabs**
   - Change: Provide a small collapsible legend: e.g., rapid vs cut vs bounds vs laser/tool marker (Visualizer) and selection vs toolpath preview vs guides (Designer).
   - Why: Color coding is not self-evident, especially for new users.
   - Done when: Users can interpret what they see without reading external docs.
   - Execution prompt: In the **Visualiser tab (sidebar or OSD)** and **Designer tab (sidebar/toolbox area or OSD)**, add a small collapsible “Legend” panel that explains the meaning of colors/lines/markers used on the canvas.

13. **Make color palette consistent and theme-friendly across Visualiser and Designer**
   - Change: Use CSS variables (accent/success/warning/error) rather than hard-coded colors.
   - Why: Hard-coded colors can clash with dark/light themes and reduce accessibility.
   - Done when: Both tabs look cohesive under theme changes and remain high-contrast.
   - Execution prompt: In the **Visualiser tab (canvas drawing + OSD)** and **Designer tab (canvas drawing + UI chrome)**, replace hard-coded colors with shared CSS variables and ensure contrast remains acceptable on dark/light themes.

14. **Add a “Grid spacing” control with the same UI pattern in both tabs**
   - Change: Provide a compact dropdown/spin control for grid spacing (unit-aware), placed in the same area on both tabs.
   - Why: Grid usefulness depends on adjustable scale.
   - Done when: Users can tune grid granularity without digging through settings.
   - Execution prompt: In the **Visualiser tab (grid controls in sidebar/OSD)** and **Designer tab (grid controls in toolbar/OSD)**, add a unit-aware “Grid spacing” control using the same widget style and placement pattern.

15. **Add a “Snap” toggle + snap strength/threshold control in Designer, mirroring Visualiser UI language**
   - Change: Add snap toggle to the same control cluster style as Visualiser’s feature toggles; include tooltip.
   - Why: Precision drawing needs snapping, and it should be explicit.
   - Done when: Snapping is discoverable, controllable, and visually indicated.
   - Execution prompt: In the **Designer tab (toolbox/controls panel)**, add a Snap toggle plus a threshold/strength control, styled like other feature toggles used in the Visualiser tab.

16. **Consolidate Visualiser sidebar toggles into grouped sections with consistent headers and spacing**
   - Change: Group toggles into sections (Toolpath, Guides, Simulation) using the same section styling used elsewhere in the app.
   - Why: A long list of checkboxes is visually noisy and harder to scan.
   - Done when: Users can quickly find toggles without reading every line.
   - Execution prompt: In the **Visualiser tab (left sidebar panel)**, regroup all toggles into clear titled sections (Toolpath/Guides/Simulation) using the app’s standard section/card styling.

17. **Add a single, consistent “Selection/Info” inspector panel pattern**
   - Change: Standardize on a right-side inspector (or bottom sheet) pattern that shows: bounds, selection dimensions, and key metadata.
   - Why: Visualiser has bounds labels; Designer has properties panel + layers; the patterns don’t match.
   - Done when: Both tabs have a consistent “info/inspector” feel (even if content differs).
   - Execution prompt: In the **Visualiser tab (right sidebar/inspector area)** and **Designer tab (properties/layers panel)**, align layout and styling to a shared “Inspector” pattern (title, sections, key/value rows).

18. **Improve Designer toolbox affordance: show tool name + shortcut on hover and add an “active tool” chip**
   - Change: Keep icon grid, but add a small label/chip in the UI that shows current tool (e.g., “Select (S)”).
   - Why: Icon-only toolbars can be opaque; users forget which mode they are in.
   - Done when: Current tool is always visible without hovering.
   - Execution prompt: In the **Designer tab (toolbox panel)**, add an always-visible “Active tool” chip/label and ensure each tool button shows name + shortcut via tooltip.

19. **Add consistent empty-state messaging**
   - Change: When no G-code is loaded (Visualizer) or no shapes exist (Designer), show a small centered empty-state with next steps (Open/import/draw).
   - Why: Blank canvases feel broken.
   - Done when: New users immediately know what to do first.
   - Execution prompt: In the **Visualiser tab (canvas area)** and **Designer tab (canvas area)**, add a centered empty-state widget shown when there’s no content, with clear next-action buttons/links.

20. **Add consistent progress + cancellation UI for expensive operations (stock removal / toolpath preview generation)**
   - Change: Display a non-blocking progress indicator (OSD or sidebar) with a cancel button.
   - Why: Long-running tasks can feel like the UI is frozen.There is an existing StatusBar hosted mechanism, add a cancel button and leverage that. 
   - Done when: Users see progress, can cancel, and the UI stays responsive.
   - Execution prompt: In the **Visualiser tab (simulation panel/OSD)** and **Designer tab (preview generation panel/OSD)**, add a non-blocking progress indicator and a Cancel action for long-running compute tasks.

21. **Make Designer status bar match Visualiser’s OSD approach (or vice-versa) and remove redundant status surfaces**
   - Change: Pick one primary status surface per tab (prefer OSD for consistency), and ensure the other doesn’t duplicate the same info.
   - Why: Multiple status areas split attention and waste space.
   - Done when: Each tab has one obvious place to look for status.
   - Execution prompt: In the **Designer tab (status bar vs OSD)**, choose a single primary status surface that matches the Visualiser tab’s status presentation and remove/condense redundant duplicated status text.

22. **Add a consistent keyboard shortcut help popover (“?”) in both tabs**
   - Change: Add a small help button that shows the most relevant shortcuts for that tab.
   - Why: Discoverability of shortcuts is a major accelerator.
   - Done when: A user can learn shortcuts in-app without opening docs.
   - Execution prompt: In the **Visualiser tab (header/OSD controls area)** and **Designer tab (header/OSD/toolbox area)**, add a “?” help button that opens a popover listing the most important shortcuts for that tab.

23. **Improve hit targets and spacing for FHD @ 125%**
   - Change: Ensure OSD buttons meet a minimum clickable size (≈28–32px) and don’t sit too close to edges; align margins between tabs.
   - Why: Small targets are hard to hit at 125% scaling.
   - Done when: OSD controls are easy to click without precision.
   - Execution prompt: In the **Visualiser tab (OSD control clusters)** and **Designer tab (OSD control clusters)**, enforce minimum button sizes and consistent padding/margins optimized for FHD @ 125% scaling.

24. **Standardize “Fit” semantics (Fit to View vs Fit to Content vs Fit to Device) and name them consistently**
   - Change: Define three explicit actions and keep naming consistent across both tabs.
   - Why: “Fit” can mean different things; ambiguity causes mistakes.
   - Done when: Users can predict each Fit action’s result from the label/tooltip alone.
   - Execution prompt: In the **Visualiser tab (view controls OSD)** and **Designer tab (view controls OSD)**, implement three distinct Fit actions with consistent naming/tooltips: Fit to Viewport, Fit to Content, and Fit to Device Working Area.

25. **Add consistent right-click context menus (Designer + Visualiser)**
   - Change: Provide a context menu with the most common actions (Fit, Reset, Copy coordinates, toggles) using the same structure on both tabs.
   - Why: Context menus are a low-friction discovery mechanism.
   - Done when: Right click provides useful, consistent actions in both tabs.
   - Execution prompt: In the **Visualiser tab (canvas right-click menu)** and **Designer tab (canvas right-click menu)**, add a matching context menu structure providing Fit/Reset/Copy-coordinates and common toggles.
