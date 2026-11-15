//! The input method (IME, IMM32): the composition turned into `Ime` events, and the place of the candidate
//! window. This module is the only one that names IMM.
//!
//! Words: the composition (preedit) is the text the input method is still building; a commit is the finished
//! text; the candidate window is the system's list of conversions, which the input method draws and we only
//! place. TSF (the newer text services) is not used: IMM32 still serves every desktop input method.

use nui_base::{Ime, observe};
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::UI::Input::Ime::{
    CANDIDATEFORM, CFS_EXCLUDE, CFS_POINT, COMPOSITIONFORM, CPS_CANCEL, GCS_COMPSTR, GCS_CURSORPOS, GCS_RESULTSTR, HIMC, IACE_DEFAULT,
    IME_COMPOSITION_STRING, ImmAssociateContextEx, ImmGetCompositionStringW, ImmGetContext, ImmNotifyIME, ImmReleaseContext,
    ImmSetCandidateWindow, ImmSetCompositionWindow, NI_COMPOSITIONSTR,
};

pub(crate) fn decode(units: &[u16]) -> String {
    String::from_utf16_lossy(units)
}

/// The number of chars before UTF-16 index `unit`, clamped to the text: the input method counts its caret in
/// UTF-16 units, the field in chars. A surrogate pair counts as one char, and an index inside a pair counts
/// the pair as not yet passed.
pub(crate) fn char_cursor(units: &[u16], unit: i32) -> usize {
    let unit = unit.clamp(0, units.len() as i32) as usize;
    char::decode_utf16(units.iter().copied()).scan(0usize, |at, r| {
        let start = *at;
        *at += r.map_or(1, char::len_utf16);
        Some((start, *at))
    })
    .take_while(|&(_, end)| end <= unit)
    .count()
}

/// The events of one `WM_IME_COMPOSITION`, in the order the field must see them: the commit first, then the
/// composition that goes on (an input method may finish one phrase and start the next in one message). An
/// update is sent whenever the composition string was asked for, even an empty one: that is how a cancel
/// reaches the field. With neither (lparam 0, the composition was cleared) the update is empty.
pub(crate) fn composition_events(result: Option<&[u16]>, comp: Option<&[u16]>, cursor_units: i32) -> Vec<Ime> {
    let mut out = Vec::new();
    if let Some(r) = result.filter(|r| !r.is_empty()) {
        out.push(Ime::Commit { text: decode(r) });
    }
    match comp {
        Some(c) => out.push(Ime::Update { text: decode(c), cursor: char_cursor(c, cursor_units) }),
        None if result.is_none() => out.push(Ime::Update { text: String::new(), cursor: 0 }),
        None => {}
    }
    out
}

/// One string of the composition, as UTF-16 units. The first call gives the length in bytes.
unsafe fn read(himc: HIMC, kind: IME_COMPOSITION_STRING) -> Vec<u16> {
    unsafe {
        let bytes = ImmGetCompositionStringW(himc, kind, None, 0);
        if bytes <= 0 {
            return Vec::new();
        }
        let mut buf = vec![0u16; bytes as usize / 2];
        let got = ImmGetCompositionStringW(himc, kind, Some(buf.as_mut_ptr().cast()), bytes as u32);
        buf.truncate((got.max(0) as usize) / 2);
        buf
    }
}

/// The events for a `WM_IME_COMPOSITION` whose flags are `lparam`.
pub(crate) unsafe fn on_composition(hwnd: HWND, lparam: isize) -> Vec<Ime> {
    unsafe {
        let flags = lparam as u32;
        if flags == 0 {
            return composition_events(None, None, 0);
        }
        let (has_result, has_comp) = (flags & GCS_RESULTSTR.0 != 0, flags & GCS_COMPSTR.0 != 0);
        if !has_result && !has_comp {
            // Only attributes or the caret moved: the string is the same.
            return Vec::new();
        }
        let himc = ImmGetContext(hwnd);
        if himc.0.is_null() {
            observe!(Warn, "ime: composition message with no input context");
            return Vec::new();
        }
        let result = has_result.then(|| read(himc, GCS_RESULTSTR));
        let comp = has_comp.then(|| read(himc, GCS_COMPSTR));
        let cursor = ImmGetCompositionStringW(himc, GCS_CURSORPOS, None, 0);
        let _ = ImmReleaseContext(hwnd, himc);
        composition_events(result.as_deref(), comp.as_deref(), cursor)
    }
}

/// Gives the window its default input context back (on), or removes it (off): with none associated, the
/// input method does nothing for this window and its indicator shows it off.
pub(crate) unsafe fn set_enabled(hwnd: HWND, on: bool) {
    unsafe {
        let _ = ImmAssociateContextEx(hwnd, HIMC::default(), if on { IACE_DEFAULT } else { 0 });
    }
}

/// Tells the input method where the caret is, in client pixels. The system's composition window is hidden
/// (we draw the composition ourselves), but its position is still what some input methods anchor to. The
/// candidate list goes under the caret, excluding the caret's own box: CFS_EXCLUDE keeps it from covering the
/// line being typed.
pub(crate) unsafe fn place(hwnd: HWND, spot_px: RECT) {
    unsafe {
        let himc = ImmGetContext(hwnd);
        if himc.0.is_null() {
            return;
        }
        let comp = COMPOSITIONFORM { dwStyle: CFS_POINT, ptCurrentPos: POINT { x: spot_px.left, y: spot_px.top }, rcArea: RECT::default() };
        let _ = ImmSetCompositionWindow(himc, &comp);
        let cand = CANDIDATEFORM { dwIndex: 0, dwStyle: CFS_EXCLUDE, ptCurrentPos: POINT { x: spot_px.left, y: spot_px.bottom }, rcArea: spot_px };
        let _ = ImmSetCandidateWindow(himc, &cand);
        let _ = ImmReleaseContext(hwnd, himc);
    }
}

/// Drops the composition in progress, with no commit.
pub(crate) unsafe fn cancel(hwnd: HWND) {
    unsafe {
        let himc = ImmGetContext(hwnd);
        if himc.0.is_null() {
            return;
        }
        let _ = ImmNotifyIME(himc, NI_COMPOSITIONSTR, CPS_CANCEL, 0);
        let _ = ImmReleaseContext(hwnd, himc);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }

    #[test]
    fn the_cursor_counts_a_surrogate_pair_as_one_char() {
        // U+2000B is two units; "a" is one.
        let units = u("\u{2000B}a");
        assert_eq!(units.len(), 3);
        assert_eq!(char_cursor(&units, 0), 0);
        // Inside the pair: never a panic, and the pair is not yet passed.
        assert_eq!(char_cursor(&units, 1), 0);
        assert_eq!(char_cursor(&units, 2), 1);
        assert_eq!(char_cursor(&units, 3), 2);
        assert_eq!(char_cursor(&units, 99), 2);
        assert_eq!(char_cursor(&units, -4), 0);
    }

    #[test]
    fn a_result_and_a_composition_in_one_message_commit_first() {
        let ev = composition_events(Some(&u("日本")), Some(&u("ご")), 1);
        assert_eq!(ev, vec![Ime::Commit { text: "日本".into() }, Ime::Update { text: "ご".into(), cursor: 1 }]);
    }

    #[test]
    fn an_empty_composition_is_still_an_update() {
        assert_eq!(composition_events(None, Some(&[]), 0), vec![Ime::Update { text: String::new(), cursor: 0 }]);
    }

    #[test]
    fn nothing_at_all_clears_the_composition() {
        assert_eq!(composition_events(None, None, 0), vec![Ime::Update { text: String::new(), cursor: 0 }]);
    }

    #[test]
    fn a_result_alone_is_a_commit_alone() {
        assert_eq!(composition_events(Some(&u("a")), None, 0), vec![Ime::Commit { text: "a".into() }]);
    }

    #[test]
    fn a_lone_surrogate_decodes_lossily() {
        assert_eq!(decode(&[0xD800, 0x61]), "\u{FFFD}a");
        assert_eq!(char_cursor(&[0xD800, 0x61], 2), 2);
    }
}
