//! The window. A stub until step 2: `lib.rs` already exports `run`, so its signature is fixed here.

use nui_base::App;

/// Opens a window for `app` and runs until it closes. Not written yet.
pub fn run(_title: &str, _app: &mut dyn App) -> windows::core::Result<()> {
    Err(windows::core::Error::new(windows::Win32::Foundation::E_NOTIMPL, "the window is step 2"))
}
