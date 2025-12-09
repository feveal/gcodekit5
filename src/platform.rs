//! Platform-specific utilities
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, Win32WindowHandle, WindowHandle, WindowsDisplayHandle,
};

/// Wrapper for Win32 RawWindowHandle
#[cfg(target_os = "windows")]
pub struct Win32ParentHandle(pub std::num::NonZeroIsize);

#[cfg(target_os = "windows")]
impl HasWindowHandle for Win32ParentHandle {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let handle = Win32WindowHandle::new(self.0);
        let raw = RawWindowHandle::Win32(handle);
        unsafe { Ok(WindowHandle::borrow_raw(raw)) }
    }
}

#[cfg(target_os = "windows")]
impl HasDisplayHandle for Win32ParentHandle {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        let handle = WindowsDisplayHandle::new();
        let raw = RawDisplayHandle::Windows(handle);
        unsafe { Ok(DisplayHandle::borrow_raw(raw)) }
    }
}

/// Initialize the window (maximize and focus)
///
/// This is a no-op for non-GUI platforms; GUI-specific behavior is implemented
/// in the UI crate's platform helpers (GTK/Win32 as appropriate).
pub fn initialize_window() {
    // For now, this helper does nothing; GUI crates should perform any
    // platform-specific window manipulation as needed.
}

/// Helper to call FileDialog::pick_file with the current foreground window as parent on Windows
pub fn pick_file_with_parent(dialog: rfd::FileDialog) -> Option<PathBuf> {
    // The UI crate will handle parent window associations if necessary. For
    // the generic platform helper we simply call pick_file() directly.
    dialog.pick_file()
}

/// Helper to call FileDialog::save_file with the current foreground window as parent on Windows
pub fn save_file_with_parent(dialog: rfd::FileDialog) -> Option<PathBuf> {
    dialog.save_file()
}

/// Helper for folder picking with the foreground window as parent on Windows
pub fn pick_folder_with_parent(dialog: rfd::FileDialog) -> Option<PathBuf> {
    dialog.pick_folder()
}

/// Invoke a closure on the main thread / event loop.
///
/// Currently this is a no-op and executes the closure immediately to keep
/// tests and non-GUI code simple. Replace this with a real main-loop
/// scheduling call if switching to a GUI event loop (glib/gtk) later.
pub fn invoke_from_event_loop<F: FnOnce() + Send + 'static>(f: F) -> Result<(), ()> {
    f();
    Ok(())
}
