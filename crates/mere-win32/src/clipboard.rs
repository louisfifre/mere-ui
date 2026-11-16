//! The system clipboard as text (CF_UNICODETEXT).

use mere_base::{Clipboard, observe};
use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL};
use windows::Win32::System::DataExchange::{CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData};
use windows::Win32::System::Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock};
use windows::Win32::System::Ole::CF_UNICODETEXT;

use crate::window::clipboard_owner;

/// The Windows clipboard. It holds nothing: every call opens the clipboard, does its work and closes it
/// again, because another program may take it between two calls.
pub struct SystemClipboard;

/// Closes the clipboard on every way out of a call.
struct Open;

impl Open {
    fn new() -> Option<Open> {
        // Without a window (tests) the owner is None; the documentation warns that setting may then fail.
        match unsafe { OpenClipboard(clipboard_owner()) } {
            Ok(()) => Some(Open),
            Err(e) => {
                observe!(Debug, "clipboard: cannot open: {e}");
                None
            }
        }
    }
}

impl Drop for Open {
    fn drop(&mut self) {
        let _ = unsafe { CloseClipboard() };
    }
}

impl Clipboard for SystemClipboard {
    fn get(&mut self) -> Option<String> {
        let _open = Open::new()?;
        unsafe {
            let handle = match GetClipboardData(CF_UNICODETEXT.0 as u32) {
                Ok(h) => h,
                Err(e) => {
                    observe!(Debug, "clipboard: no text: {e}");
                    return None;
                }
            };
            let global = HGLOBAL(handle.0);
            let p = GlobalLock(global) as *const u16;
            if p.is_null() {
                observe!(Debug, "clipboard: the text could not be locked");
                return None;
            }
            let mut len = 0;
            while *p.add(len) != 0 {
                len += 1;
            }
            let text = String::from_utf16_lossy(std::slice::from_raw_parts(p, len));
            let _ = GlobalUnlock(global);
            Some(text)
        }
    }

    fn set(&mut self, text: &str) {
        let wide: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
        unsafe {
            let Ok(global) = GlobalAlloc(GMEM_MOVEABLE, wide.len() * 2) else {
                observe!(Warn, "clipboard: no memory for {} bytes", wide.len() * 2);
                return;
            };
            let p = GlobalLock(global) as *mut u16;
            if p.is_null() {
                let _ = GlobalFree(Some(global));
                observe!(Warn, "clipboard: the memory could not be locked");
                return;
            }
            std::ptr::copy_nonoverlapping(wide.as_ptr(), p, wide.len());
            let _ = GlobalUnlock(global);
            let Some(_open) = Open::new() else {
                let _ = GlobalFree(Some(global));
                observe!(Warn, "clipboard: could not open it to set the text");
                return;
            };
            let _ = EmptyClipboard();
            // On success the system owns the memory; on failure it is still ours to free.
            if let Err(e) = SetClipboardData(CF_UNICODETEXT.0 as u32, Some(HANDLE(global.0))) {
                let _ = GlobalFree(Some(global));
                observe!(Warn, "clipboard: setting the text failed: {e}");
            }
        }
    }
}
