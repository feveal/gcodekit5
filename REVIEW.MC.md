# Machine Control Tab — Aesthetics & Usability Recommendations

Each item below is **separately implementable**, can be done **out of sequence**, and is **complete on its own**.

1. **Replace tick-based Paned auto-resize with stable, user-respectful sizing**
   - Change: Remove `add_tick_callback` resizing (both inner + outer paned) and set initial positions once (on realize/size-allocate), then let the user adjust.
   - Why: Continuous tick callbacks can fight the user, cause jitter, and waste CPU.
   - Done when: Divider positions no longer “snap back” while dragging; initial layout still defaults to ~20% sidebar / ~70–30 main-console.

2. **Make section headers consistent and visually lightweight**
   - Change: Replace `Frame::new(Some("..."))` titles with a uniform header row (Label + optional icon) and use `ListBox`/`PreferencesGroup`-like spacing.
   - Why: Current Frames create heavy visual boxes that compete with primary controls.
   - Done when: Connection/Transmission/State/WCS look like a cohesive sidebar system rather than four separate boxed widgets.

3. **Add tooltips + accessible labels for icon-only controls**
   - Change: Add tooltips for `refresh_btn`, jog arrow buttons, e-stop image button, and the DRO “⊙” zero buttons; ensure accessible names are set.
   - Why: Icon-only controls are ambiguous, especially for new users.
   - Done when: Hovering shows clear text (e.g., “Refresh ports”, “Jog X+”, “Zero X (G92 X0)”, “Emergency stop (soft reset)”).

4. **Rename and clarify “Reset (G53)” in Work Coordinates**
   - Change: Rename button to what it actually does (e.g., “Use G53 (Machine Coordinates)” or “Temporary G53 Move Mode”) and/or replace with a short explanation label.
   - Why: “Reset (G53)” reads like a destructive reset but the command is simply `G53`.
   - Done when: Button text matches behavior; users can predict outcome without reading docs.

5. **Show the active WCS (G54–G59) with a persistent visual state**
   - Change: Turn the WCS buttons into toggle/radio buttons (single-active group) and apply a CSS class (e.g., `active`) to the current selection.
   - Why: Users need immediate confirmation of which coordinate system is active.
   - Done when: Exactly one WCS is visually highlighted at all times (while connected).

6. **Add a “Work Coordinates (WPos)” section and display both WPos and MPos explicitly**
   - Change: Current DRO is driven by `MPos`. Add a parallel display for `WPos` (or make a toggle to switch the DRO between MPos/WPos).
   - Why: Users typically jog/zero in work coordinates; mixing concepts causes mistakes.
   - Done when: UI clearly distinguishes machine vs work coordinates and shows both without needing the StatusBar.

7. **Fix axis labeling (DRO) to avoid ambiguity (“X” vs “MachX” / “WorkX”)**
   - Change: Prefix axis labels with context (Mach X / Work X) or add a small header line above the DRO that states which coordinate system is shown.
   - Why: “X/Y/Z” alone is unclear once both coordinate systems are visible.
   - Done when: A screenshot cannot be misinterpreted regarding coordinate system.

8. **Make the “Zero” affordance safer and clearer**
   - Change: Replace the “⊙” button with a labeled icon button (e.g., crosshair icon + “Zero”) and add confirmation for “Zero All Axes”.
   - Why: Zeroing is destructive and currently too easy to click accidentally.
   - Done when: Accidental zeroing is less likely; dangerous actions require explicit intent.

9. **Add “Go To Zero” / “Go To Work Origin” quick action**
   - Change: Add a button that executes a safe move to X0 Y0 (and optionally Z safe height first).
   - Why: It’s a common workflow step and currently requires manual commands.
   - Done when: Users can return to work origin from the Machine Control tab without typing G-code.

10. **Add feed rate control for jogging (and show current jogging feed)**
   - Change: Replace the hard-coded `F2000` jog feed with a user-adjustable control (spin/slider) and show the selected value near the jog pad.
   - Why: Jog speed needs to vary by machine size, axis, and safety context.
   - Done when: User can set jog feed; jog commands use that value; UI shows it clearly.

11. **Make step size unit-aware (Metric/Imperial) and label it dynamically**
   - Change: Replace hard-coded “Step (mm)” + metric numbers with unit-aware values driven by settings (e.g., show “Step (in)” when Imperial; re-render labels on settings changes).
   - Why: Current UI can show DRO in inches while step still claims mm.
   - Done when: Step label and values reflect current measurement system; internal units remain correct.

12. **Offer more step size presets (including very small + very large) in a compact control**
   - Change: Replace 4 toggle buttons with a `DropDown`/ComboBox (e.g., 0.001, 0.01, 0.1, 1, 10, 100) using `JogStepSize` presets already defined elsewhere.
   - Why: 4 buttons are visually bulky and insufficient for many machines.
   - Done when: Step selection is compact, scalable, and includes micro-steps.

13. **Improve jog pad affordances: add axis labels and direction semantics**
   - Change: Add small labels around the XY pad (X-, X+, Y-, Y+), and optionally add a subtle compass/coordinate indicator.
   - Why: Arrow icons alone don’t indicate machine coordinate conventions (and can be inverted depending on setup).
   - Done when: Users can tell the commanded axis/direction without guessing.

14. **Add keyboard jog shortcuts (optional, clearly indicated)**
   - Change: Implement configurable shortcuts (numpad/arrow keys) and show hints (“8/2/4/6/9/3”) in the UI near the jog pad.
   - Why: Keyboard jogging is faster and improves accessibility.
   - Done when: Jogging works from keyboard when the tab is focused; hints can be toggled off.

15. **Make the e-stop control more explicit and consistent with GNOME/GTK destructive patterns**
   - Change: Keep the big button but add a text label (“E‑STOP”) and show what it does (soft reset Ctrl‑X) via tooltip; ensure it is the most prominent destructive action.
   - Why: Image-only e-stop can be unclear and theming may reduce contrast.
   - Done when: e-stop is clearly labeled, high-contrast, and self-explanatory.

16. **Differentiate Stop vs E‑Stop behavior in UI copy and styling**
   - Change: Add explanatory microcopy: Stop = abort stream; E‑Stop = controller reset. Ensure both are clearly distinct.
   - Why: Users may assume both do the same thing; wrong choice can lose state.
   - Done when: A user understands consequences without reading documentation.

17. **Add a connection status strip in the sidebar (port + baud + state)**
   - Change: Add a compact, always-visible status row showing selected port, baud, and “Connected/Disconnected/Alarm”.
   - Why: State is currently spread across controls; quick scanning is harder.
   - Done when: One glance reveals connection + machine state.

18. **Improve “Machine State” presentation with color + icon + secondary details**
   - Change: Replace the single large text label with: state badge (Idle/Run/Hold/Alarm) + small secondary lines (feed, spindle, buffer).
   - Why: State alone is insufficient; the app already parses feed/spindle/buffer.
   - Done when: Machine State block provides actionable operational awareness.

19. **Visually separate “setup” actions from “run job” actions**
   - Change: Group sidebar actions into two visual groups: Setup (Connect/Home/Unlock/WCS/Zero) and Job (Send/Pause/Resume/Stop).
   - Why: Reduces accidental job starts and improves mental model.
   - Done when: Users can find setup vs job controls instantly.

20. **Add “disabled reason” feedback when controls are insensitive**
   - Change: When disconnected or in ALARM, show a small hint label (e.g., “Connect to enable jogging” / “Unlock required”).
   - Why: Disabled UI without explanation feels broken.
   - Done when: Users always know what prerequisite is missing.

21. **Make Device Console panel resizable/collapsible with a clear header**
   - Change: Add a header row with title (“Device Console”), clear button, and a collapse toggle; remember collapse state.
   - Why: Console competes for horizontal space; users sometimes need more DRO/jog space.
   - Done when: Console can be hidden quickly; layout remains stable.

22. **Add quick “Clear Console” and “Copy Last Error” actions**
   - Change: Provide buttons in the console header to clear logs and copy last non-status error line.
   - Why: Improves debugging and reduces noise during long sessions.
   - Done when: Users can clean the view and quickly share relevant error context.

23. **Use gettext (`t!()`) for all visible strings in Machine Control**
   - Change: Wrap labels like “Connection”, “Transmission”, “Machine State”, “Work Coordinates”, “Zero All Axes”, etc. with `t!()`.
   - Why: The app already supports translations; this tab currently uses raw strings.
   - Done when: `scripts/update-po.sh` extracts these strings and PO files can translate them.

24. **Normalize spacing/margins for a more modern GTK look**
   - Change: Remove hard-coded width requests where possible (e.g., sidebar 200px, DRO 600px) and rely on GTK layout + consistent margins.
   - Why: Fixed sizing breaks on small/large displays and looks less native.
   - Done when: Tab scales gracefully from small laptop screens to large monitors.

25. **Add a small “Safety checklist” warning box shown only when connected**
   - Change: Use the existing `.warning-box` styling to show a short reminder (e.g., “Confirm clear path before homing/jogging”).
   - Why: Reinforces safe operation without being intrusive.
   - Done when: Warning appears contextually and can be dismissed per session.
