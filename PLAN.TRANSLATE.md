# Plan: Multi-language Translation Support (i18n/l10n)

**Goal**: Implement switchable multi-language support for GCodeKit5 using standard GNOME/GTK4 conventions (`gettext`), allowing users to select their preferred language via the application settings.

## 1. Infrastructure Setup

### 1.1 Dependencies
- [ ] Add `gettext-rs` (or `gettext-sys`) to `Cargo.toml` dependencies for runtime translation loading.
- [ ] Add `tr!` macro support (often via `gettextrs` or a wrapper crate like `gettext-macros` if preferred, though standard `gettext` calls are more common in Rust/GTK apps).
- [ ] Ensure `glib` and `gio` features are enabled for locale handling.

### 1.2 Directory Structure
- [ ] Create `po/` directory in the project root for translation files.
- [ ] Create `po/POTFILES.in` listing all source files containing translatable strings.
- [ ] Create `po/LINGUAS` listing supported languages.

### 1.3 Build System Integration
- [ ] Update `build.rs` to compile `.po` files into `.mo` files during the build process.
- [ ] Configure the install step to place `.mo` files in the standard system locale directory (e.g., `/usr/share/locale/<lang>/LC_MESSAGES/gcodekit5.mo`) or a local resource path for Flatpak/development.
- [ ] Ensure `meson` or `make` scripts (if used for packaging) handle `msgfmt` and installation.

## 2. Code Instrumentation (Internationalization - i18n)

### 2.1 String Marking
- [ ] Define a helper macro/function (e.g., `gettext` or `_()`) to wrap translatable strings.
- [ ] Audit codebase and wrap all user-facing strings in UI code (GTK widgets, labels, tooltips).
  - *Example*: `label.set_text(&gettext("Connect"));`
- [ ] Handle formatted strings using `fl!` (from `fluent`) or `format!` with localized templates. *Note: `gettext` uses C-style formatting, Rust might need careful handling or `ngettext` for plurals.*
- [ ] Mark static strings/constants that need translation at runtime.

### 2.2 UI Files (.ui / .blp)
- [ ] If using `.ui` files, ensure `translatable="yes"` attribute is set on text properties.
- [ ] Ensure the build process extracts strings from `.ui` files into the `.pot` template.

## 3. Translation Management (Localization - l10n)

### 3.1 Template Generation
- [ ] Create a script or `cargo` task to run `xgettext` and generate/update the `gcodekit5.pot` template file.
- [ ] Configure `xgettext` to recognize Rust string markers.

### 3.2 Initial Translations
- [ ] Create initial `.po` files for target languages (e.g., `es.po`, `de.po`, `fr.po`) from the template.
- [ ] Populate a few key strings (Menu items, Buttons) to test the pipeline.

## 4. Runtime Language Switching

### 4.1 Settings Integration
- [ ] Add a "Language" dropdown to the **General** tab in `Config Settings`.
- [ ] Populate dropdown with available system locales or a predefined list of supported languages.
- [ ] Persist the selected language code (e.g., `en_US`, `de_DE`) in the application settings (`gcodekit5-settings`).

### 4.2 Locale Application
- [ ] Implement logic in `main.rs` startup to read the saved language setting.
- [ ] Call `gettextrs::setlocale` and `gettextrs::bindtextdomain` to initialize the translation system before UI construction.
- [ ] **Dynamic Switching**:
  - *Challenge*: GTK4 does not natively support dynamic language switching without restart for all widgets.
  - *Strategy*:
    - **Option A (Restart)**: Prompt user to restart the application when language changes (Standard GNOME behavior).
    - **Option B (Dynamic)**: Emit a `LanguageChanged` event. All UI components subscribe to this and re-call `set_text()` on their widgets. (More complex, but better UX).
    - *Decision*: Start with **Option A (Restart)** for reliability, then explore Option B for specific panels.

## 5. Packaging and Distribution

### 5.1 Flatpak
- [ ] Update `flatpak/org.gcodekit.GCodeKit5.yml` (or manifest) to include the `po` directory processing.
- [ ] Ensure `locales` are properly exported in the Flatpak bundle.

### 5.2 OS Integration
- [ ] Verify `.mo` files are installed to `/usr/share/locale` on Linux.
- [ ] For Windows/macOS, ensure `.mo` files are bundled in a `share/locale` folder relative to the executable and `bindtextdomain` points to it.

## 6. Verification

### 6.1 Testing
- [ ] Verify fallback to English if a translation is missing.
- [ ] Verify correct handling of Unicode characters and text direction (RTL support if needed).
- [ ] Test language persistence across restarts.

### 6.2 Documentation
- [ ] Update `CONTRIBUTING.md` with instructions for translators (how to update `.po` files).
