# REVIEW.DEVICECONFIG.md

## Scope
This review covers the **Device Config** tab UI (GRBL settings + embedded device info) and proposes a more space-efficient, consistent, and safer configuration workflow.

**Primary implementation locations:**
- Tab wiring: `crates/gcodekit5-ui/src/gtk_app.rs` (stack child `"config"`)
- Device Config panel: `crates/gcodekit5-ui/src/ui/gtk/config_settings.rs` (`ConfigSettingsView`)
- Embedded device info: `crates/gcodekit5-ui/src/ui/gtk/device_info.rs` (`DeviceInfoView`)
- Styling: `crates/gcodekit5-ui/src/ui/gtk/style.css` ("Device Config & Info Views")

---

## Key observations (current)
- The tab is implemented as a wide **2-column horizontal Box** with a fixed-width left panel (`width_request(320)`) and a table-like `ListBox` on the right.
- The settings list is a “fake table” (manual header row + fixed-width labels). This is space-heavy, not sortable, not resizable, and doesn’t scale well.
- Editing a value requires opening a modal `gtk::Dialog`; there’s no inline validation, ranges, or unit handling beyond a label.
- “Restore” is currently a placeholder and lacks guardrails (preview/diff/confirm).
- Retrieval is implemented by scraping console log output after sending `$$`; status/progress is basic and not cancellable.

---

## Recommendations (actionable)

### 1) Replace the ListBox "table" with a real Gtk4 `ColumnView` (sortable, resizable)
**Change:** Use `gtk::ColumnView` + a list model (`gio::ListStore`) for settings rows, with columns:
- `$ID`
- Name
- Value (editable)
- Unit
- Category
- Description (optional / collapsible)

**Why:** Makes the settings view dramatically more space-efficient and functional (sorting, resizing, better alignment).

**Done when:** User can resize columns, sort by ID/Category/Name, and the list remains readable at 20–25% sidebar widths.

**Execution prompt:** In `crates/gcodekit5-ui/src/ui/gtk/config_settings.rs`, replace the `ListBox` + manual header row with a `gtk::ColumnView` backed by a model built from `ConfigSettingRow`.

---

### 2) Add "compact" mode: hide Description column by default, show on selection
**Change:** In compact mode, show only ID/Name/Value/Unit; show Description in a details area below (or on the right) for the selected setting.

**Why:** The Description column is the biggest horizontal space hog.

**Done when:** The list is readable in ~320px width while still allowing access to descriptions.

**Execution prompt:** In the new `ColumnView`, hide the Description column and render description in a separate panel bound to the current selection.

---

### 3) Inline editing for Value with validation + read-only affordance
**Change:** Make Value editable inline (e.g., `EditableLabel`, `Entry`, or `SpinButton` depending on setting metadata).
- If value invalid → show error styling and block sending to device.
- If `read_only` → render disabled + lock icon.

**Why:** Modal edit dialogs slow down iteration and hide context.

**Done when:** Changing `$110` can be done directly in the list, with obvious errors and no crashes.

**Execution prompt:** In `config_settings.rs`, store per-row editing state and commit changes via `communicator` on apply/enter, with `.entry-invalid` CSS.

---

### 4) Add a "Pending changes" bar with Apply / Revert / Copy-to-clipboard
**Change:** Track edits locally first; only send to device on **Apply**.
- Provide `Apply`, `Revert`, `Copy changes` actions.

**Why:** Prevents accidental device configuration drift and enables multi-setting edits.

**Done when:** User can change 3 settings and apply them in one operation.

**Execution prompt:** Extend `ConfigSettingsView` to track dirty rows and show a bottom action bar when any row differs from the last retrieved snapshot.

---

### 5) Retrieval should parse responses from a structured channel, not scrape console text
**Change:** Prefer a dedicated status/settings pipeline (or a structured callback from the polling loop) rather than substring matching the console log.

**Why:** Console scraping is brittle and can fail depending on logging, timing, or user actions.

**Done when:** Retrieving settings works consistently even if the console is cleared or spammed.

**Execution prompt:** In `ConfigSettingsView::retrieve_settings()`, replace "read from DeviceConsoleView log" with a settings-response accumulator fed by the same source that handles device replies.

---

### 6) Add non-blocking progress + cancel for Retrieve / Restore
**Change:** Show progress (e.g. “Receiving settings… 18/34”) and a Cancel button.

**Why:** Retrieval/restore can be slow; users need reassurance and control.

**Done when:** Cancel stops the operation and the UI stays responsive.

**Execution prompt:** Reuse the existing status/progress patterns from Visualizer/Designer (status bar hosted mechanism) for this tab; add a cancellation token checked in the polling callback.

---

### 7) Make toolbar actions more explicit + safer (group and label)
**Change:** Replace the current flat button row (`Retrieve/Save/Load/Restore`) with grouped sections:
- **Device**: Retrieve, Restore
- **File**: Import, Export
- **Tools**: Copy config, Diff

Add icons and tooltips consistently.

**Why:** The current row reads like generic file operations and doesn’t communicate risk.

**Done when:** "Restore" is clearly a potentially destructive action (accent/warning styling + confirm).

**Execution prompt:** In `ConfigSettingsView::new()`, replace `Button::with_label(...)` with icon+label buttons and apply consistent CSS classes.

---

### 8) Add a Diff/Preview step for Restore-to-device
**Change:** Before restoring settings to the device, show:
- what settings will change
- old value → new value
- a checkbox “I understand this changes machine configuration”

**Why:** Device settings can permanently affect motion behavior.

**Done when:** Restore requires explicit confirmation and provides a diff list.

**Execution prompt:** Implement `restore_to_device()` to open a dialog listing changes derived from (a) last retrieved snapshot, (b) current local edits/imported file.

---

### 9) Improve the left "Device Info" panel density and visual hierarchy
**Change:** Make the left panel smaller and denser:
- Replace emoji icons with symbolic icons
- Use compact rows (label + value on one line)
- Move actions into a small header toolbar (Refresh / Copy)

**Why:** The left panel currently consumes a lot of space and feels like a separate page.

**Done when:** Device info fits comfortably in ~240px width and doesn’t dominate the config list.

**Execution prompt:** In `device_info.rs`, rework the sidebar from a tall stacked layout to compact rows and a small action header.

---

### 10) Use a `Paned` with persistence and a hide/unhide affordance for the left info panel
**Change:** Replace the outer `Box(Orientation::Horizontal)` with `gtk::Paned`:
- initial left size ~20% width
- allow user drag resize
- persist position
- allow hide/unhide (like DeviceConsole/Visualizer sidebar patterns)

**Why:** Better control of space on different screen sizes.

**Done when:** Left panel can be collapsed and restored, and user resizing persists.

**Execution prompt:** In `ConfigSettingsView::new()`, use a `Paned` and persist position using the same approach as other sidebars.

---

### 11) Add quick filters as toggles (Changed / Read-only / Homing / Motion / Spindle)
**Change:** Add a compact filter strip with toggle buttons:
- Changed
- Read-only
- Motion (Steps/Rate/Accel)
- Homing
- Spindle/Laser

**Why:** The Category dropdown is useful but slower than one-click toggles.

**Done when:** Filtering to "Changed" shows only modified settings.

**Execution prompt:** In `config_settings.rs`, extend `apply_filter()` to incorporate toggle states.

---

### 12) Add import/export format clarity + validation feedback
**Change:** When importing/exporting:
- show file path in status
- validate schema/version
- show how many settings were loaded
- highlight conflicts/unknown settings

**Why:** Users need confidence the file is correct and complete.

**Done when:** Import of an invalid file shows a clear error and doesn’t silently succeed.

**Execution prompt:** In `save_to_file()` / `load_from_file()`, implement actual serialization/deserialization and surface results via the status label + a details dialog on error.
