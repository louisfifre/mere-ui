//! The page's ease and the slide.

use mere_ui::{SLIDE_SECONDS, Slide, ease};

fn near(a: f32, b: f32, tol: f32) {
    assert!((a - b).abs() <= tol, "{a} != {b}");
}

#[test]
fn ease_ends_and_middle() {
    assert_eq!(ease(0.0), 0.0);
    near(ease(1.0), 1.0, 1e-6);
    // cubic-bezier(.25,.1,.25,1) at x = 0.5 is 0.8024.
    near(ease(0.5), 0.8024, 1e-3);
}

#[test]
fn ease_is_monotonic() {
    let mut last = -1.0;
    for i in 0..=100 {
        let v = ease(i as f32 / 100.0);
        assert!(v >= last, "{i}: {v} < {last}");
        last = v;
    }
}

#[test]
fn a_slide_at_half_time_stands_at_the_eases_value() {
    let mut s = Slide::settled(false);
    s.toward(true, 10.0, false);
    near(s.value(10.0 + SLIDE_SECONDS / 2.0, false), 0.8024, 1e-3);
    assert!(s.moving(10.1, false));
    assert_eq!(s.value(11.0, false), 1.0);
    assert!(!s.moving(11.0, false));
}

#[test]
fn reduced_motion_jumps() {
    let mut s = Slide::settled(false);
    s.toward(true, 10.0, true);
    assert_eq!(s.value(10.0, true), 1.0);
    assert!(!s.moving(10.0, true));
}

#[test]
fn reversing_midway_starts_from_the_current_value() {
    let mut s = Slide::settled(false);
    s.toward(true, 10.0, false);
    let mid = s.value(10.11, false);
    s.toward(false, 10.11, false);
    near(s.value(10.11, false), mid, 1e-6);
    assert_eq!(s.value(11.0, false), 0.0);
    // Asking for what it already heads for changes nothing.
    let before = s;
    s.toward(false, 10.2, false);
    assert_eq!(s, before);
}
