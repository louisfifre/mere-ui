//! Labels measure and cut text by cells, as the fields do: a wide character takes two.

use nui_ui::{TextLook, Tone, cells_of, ellipsize, text_width, wrap_lines};

fn body() -> TextLook {
    TextLook::body(Tone::Text)
}

#[test]
fn a_wide_string_is_measured_by_its_cells() {
    let s = "\u{65e5}\u{672c}\u{8a9e}";
    assert_eq!(cells_of(s), 6);
    assert!((body().width(s) - 6.0 * body().width("a")).abs() < 1e-4);
    assert!((text_width(s, 13.0) - 6.0 * text_width("a", 13.0)).abs() < 1e-4);
}

#[test]
fn ellipsis_never_overflows_with_wide_glyphs() {
    let s = "\u{65e5}\u{672c}\u{8a9e}\u{306e}\u{540d}\u{524d}\u{3067}\u{3059}";
    for tenth in 20..160 {
        let w = tenth as f32 * 1.0;
        let cut = ellipsize(s, body(), w);
        assert!(body().width(&cut) <= w + 0.01 || cut == "\u{2026}", "{w}: {cut} is {}", body().width(&cut));
    }
}

#[test]
fn wrapping_counts_cells() {
    let step = body().width("a");
    for line in wrap_lines("\u{65e5}\u{672c}\u{8a9e}\u{306e}\u{540d}\u{524d}", body(), step * 5.0) {
        assert!(cells_of(&line) <= 5, "{line}");
    }
}
