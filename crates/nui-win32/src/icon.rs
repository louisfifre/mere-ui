//! The window's icon, the one the taskbar and the Alt+Tab list show: an RGBA square turned into the
//! system's icon handle.

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::Graphics::Gdi::{CreateBitmap, DeleteObject, HGDIOBJ};
use windows::Win32::UI::WindowsAndMessaging::{CreateIconIndirect, DestroyIcon, HICON, ICON_BIG, ICON_SMALL, ICONINFO, SendMessageW, WM_SETICON};
use windows::core::Result;

/// A square picture, one byte each for red, green, blue and alpha, rows from the top. The alpha is straight
/// (colours not multiplied by it), as a PNG decoder gives it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Icon {
    pub rgba: Vec<u8>,
    pub side: u32,
}

impl Icon {
    /// True when the bytes are exactly a `side` by `side` picture, and not empty.
    pub fn is_valid(&self) -> bool {
        self.side > 0 && self.side <= 256 && self.rgba.len() == (self.side as usize).pow(2) * 4
    }
}

/// The two bitmaps an icon is made of. The colour one is BGRA, the order the system's bitmaps use, with the
/// alpha straight, which is what the system reads in a 32-bit icon. The mask has a 1 where the pixel is fully
/// transparent; a 32-bit icon with alpha ignores it, but an older path that does not still draws the shape.
pub(crate) fn planes(icon: &Icon) -> (Vec<u8>, Vec<u8>) {
    let side = icon.side as usize;
    let mut bgra = Vec::with_capacity(icon.rgba.len());
    for px in icon.rgba.chunks_exact(4) {
        bgra.extend_from_slice(&[px[2], px[1], px[0], px[3]]);
    }
    // A monochrome bitmap's rows are padded to a whole 16-bit word.
    let row = side.div_ceil(16) * 2;
    let mut mask = vec![0u8; row * side];
    for y in 0..side {
        for x in 0..side {
            if icon.rgba[(y * side + x) * 4 + 3] == 0 {
                mask[y * row + x / 8] |= 0x80 >> (x % 8);
            }
        }
    }
    (bgra, mask)
}

/// The icon handle for a picture. The caller owns it and gives it back with `DestroyIcon`.
pub(crate) fn make(icon: &Icon) -> Result<HICON> {
    let (bgra, mask) = planes(icon);
    let side = icon.side as i32;
    unsafe {
        let colour = CreateBitmap(side, side, 1, 32, Some(bgra.as_ptr().cast()));
        let and = CreateBitmap(side, side, 1, 1, Some(mask.as_ptr().cast()));
        let info = ICONINFO { fIcon: true.into(), xHotspot: 0, yHotspot: 0, hbmMask: and, hbmColor: colour };
        let made = CreateIconIndirect(&info);
        // The icon keeps its own copy of both bitmaps.
        let _ = DeleteObject(HGDIOBJ(colour.0));
        let _ = DeleteObject(HGDIOBJ(and.0));
        made
    }
}

/// Sets the window's big icon (taskbar, Alt+Tab) and small one (the corner of a system bar). One handle
/// serves both: the system scales it to the size each asks for.
pub(crate) fn set(hwnd: HWND, icon: HICON) {
    unsafe {
        SendMessageW(hwnd, WM_SETICON, Some(WPARAM(ICON_BIG as usize)), Some(LPARAM(icon.0 as isize)));
        SendMessageW(hwnd, WM_SETICON, Some(WPARAM(ICON_SMALL as usize)), Some(LPARAM(icon.0 as isize)));
    }
}

pub(crate) fn destroy(icon: HICON) {
    unsafe {
        let _ = DestroyIcon(icon);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_by_two() -> Icon {
        // red opaque, green half, blue transparent / white opaque
        Icon { rgba: vec![255, 0, 0, 255, 0, 255, 0, 128, 0, 0, 255, 0, 255, 255, 255, 255], side: 2 }
    }

    #[test]
    fn the_colour_plane_is_bgra_with_the_alpha_untouched() {
        let (bgra, _) = planes(&two_by_two());
        assert_eq!(bgra, vec![0, 0, 255, 255, 0, 255, 0, 128, 255, 0, 0, 0, 255, 255, 255, 255]);
    }

    #[test]
    fn the_mask_marks_only_the_fully_transparent_pixels_and_pads_rows_to_a_word() {
        let (_, mask) = planes(&two_by_two());
        // Two rows of two bytes. The blue pixel has alpha 0 and sits in row 1 at x 0.
        assert_eq!(mask, vec![0, 0, 0b1000_0000, 0]);
        let wide = Icon { rgba: vec![0; 20 * 20 * 4], side: 20 };
        let (_, m) = planes(&wide);
        assert_eq!(m.len(), 4 * 20, "20 bits a row need two words");
        assert_eq!(m[0], 0xff);
        assert_eq!(m[2], 0xf0);
    }

    #[test]
    fn only_an_exact_square_is_valid() {
        assert!(two_by_two().is_valid());
        assert!(!Icon { rgba: vec![0; 15], side: 2 }.is_valid());
        assert!(!Icon { rgba: vec![], side: 0 }.is_valid());
        assert!(!Icon { rgba: vec![0; 4 * 3], side: 2 }.is_valid());
    }

    #[test]
    fn the_system_accepts_the_icon() {
        let h = make(&two_by_two()).expect("an icon handle");
        assert!(!h.is_invalid());
        destroy(h);
    }
}
