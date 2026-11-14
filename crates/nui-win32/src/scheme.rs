//! The system's light or dark setting, which the page reads as `prefers-color-scheme`.

use windows::Win32::System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RegGetValueW};
use windows::core::w;

/// True when the user chose dark for applications. A missing value or any error counts as light, which is
/// how Chrome reads it: the value only exists once the user has opened that setting.
pub fn system_dark() -> bool {
    let mut data: u32 = 1;
    let mut size = size_of::<u32>() as u32;
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
            w!("AppsUseLightTheme"),
            RRF_RT_REG_DWORD,
            None,
            Some(&mut data as *mut u32 as *mut std::ffi::c_void),
            Some(&mut size),
        )
    };
    status.is_ok() && data == 0
}

/// WM_SETTINGCHANGE carries the name of what changed; the light/dark switch arrives as "ImmersiveColorSet".
/// Other names (a policy, the wallpaper) must not cost a registry read and a possible redraw.
pub(crate) fn is_scheme_change(setting: &[u16]) -> bool {
    setting.iter().copied().eq("ImmersiveColorSet".encode_utf16())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }

    #[test]
    fn only_the_colour_set_name_is_a_scheme_change() {
        assert!(is_scheme_change(&wide("ImmersiveColorSet")));
        assert!(!is_scheme_change(&wide("Policy")));
        assert!(!is_scheme_change(&[]));
    }
}