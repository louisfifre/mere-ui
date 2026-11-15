//! `.setting .field` holding a number (preview.html lines 123 and 170): typed, or moved a step with the arrow
//! keys. The field itself is `field.rs`; this is what is particular to numbers.

use nui_layout::Dim;

use crate::edit::Edit;
use crate::field::{Extra, FieldResp, Shape, Target};
use crate::ui::Ui;

/// The `min`, `max` and `step` of the page's `known` table (line 655), or `ANY` for a field without one.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct NumberSpec {
    pub min: f64,
    pub max: f64,
    pub step: f64,
}

impl NumberSpec {
    pub const ANY: NumberSpec = NumberSpec { min: f64::NEG_INFINITY, max: f64::INFINITY, step: 1.0 };
}

/// How a number field sits: `Default` is today's `number` (as wide as its text, right-aligned, 3 px padding).
#[derive(Copy, Clone, Debug)]
pub struct NumberOpts {
    pub width: Dim,
    /// The value hugs the right edge; false starts it at the left padding.
    pub right: bool,
    /// Vertical padding with the border (the launcher's 3px + 1px is 4.0).
    pub pad_v: f32,
    pub disabled: bool,
}

impl Default for NumberOpts {
    fn default() -> Self {
        NumberOpts { width: Dim::Auto, right: true, pad_v: 3.0, disabled: false }
    }
}

/// Rust's shortest round-trip `Display` gives what JS `String(number)` gives for the page's values; -0 is
/// shown as "0", as JS does. Used for what a number field shows and for its buffer.
pub(crate) fn js_number(v: f64) -> String {
    let v = if v == 0.0 { 0.0 } else { v };
    format!("{v}")
}

/// The page's `input` event: the value follows the buffer as soon as it parses, unclamped. True when the
/// value changed.
pub(crate) fn live(buffer: &str, v: &mut f64) -> bool {
    match buffer.trim().parse::<f64>() {
        Ok(p) if p.is_finite() && p != *v => {
            *v = p;
            true
        }
        _ => false,
    }
}

/// The page's `change` event: the value is clamped to [min, max]. A buffer that is not a number puts back
/// the value the field had when it gained focus (the page falls back to the pack's value instead: a
/// documented difference). True when the value changed.
pub(crate) fn commit(buffer: &str, v: &mut f64, spec: NumberSpec, at_focus: f64) -> bool {
    let next = match buffer.trim().parse::<f64>() {
        Ok(p) if p.is_finite() => p.clamp(spec.min, spec.max),
        _ => at_focus,
    };
    let changed = next != *v;
    *v = next;
    changed
}

/// Digits after the decimal point of `x` as it is written, at most 10.
fn decimals(x: f64) -> usize {
    format!("{x}").split_once('.').map_or(0, |(_, d)| d.len()).min(10)
}

/// ArrowUp or ArrowDown: the next multiple of `step` above or below the value, counted from `min` (as the
/// browser''s `stepUp` does from an off-step value), clamped, and rounded to the digits `step` and `min` have
/// so that 0.5 + 21 x 0.025 prints 1.025 and not 1.0250000000000001. The buffer follows. True when the value
/// changed.
pub(crate) fn step(v: &mut f64, spec: NumberSpec, up: bool, edit: &mut Edit) -> bool {
    let base = if spec.min.is_finite() { spec.min } else { 0.0 };
    let q = (*v - base) / spec.step;
    let n = if up { (q + 1e-9).floor() + 1.0 } else { (q - 1e-9).ceil() - 1.0 };
    let mut next = (base + n * spec.step).clamp(spec.min, spec.max);
    let d = decimals(spec.step).max(if spec.min.is_finite() { decimals(spec.min) } else { 0 });
    let scale = 10f64.powi(d as i32);
    next = (next * scale).round() / scale;
    let changed = next != *v;
    *v = next;
    *edit = Edit::new(&js_number(next));
    changed
}

impl Ui {
    /// A number field: a hairline box with the value at its right. Typing changes `v` live; Enter, Tab or a
    /// press elsewhere clamps it to the spec; Escape puts back what it was when the field got the keyboard;
    /// Up and Down move it a step. While the field has the keyboard it ignores an outside change to `v`.
    pub fn number(&mut self, key: &str, v: &mut f64, spec: NumberSpec) -> FieldResp {
        self.number_with(key, v, spec, NumberOpts::default())
    }

    /// `number` with the box's own look: the launcher's `.memory input` is 8ch wide and right-aligned, its
    /// pairs fill their column and start at the left, and all have a vertical padding of 3 px plus the border.
    pub fn number_with(&mut self, key: &str, v: &mut f64, spec: NumberSpec, opts: NumberOpts) -> FieldResp {
        let extra = Extra { pad_v: Some(opts.pad_v), right: Some(opts.right), disabled: opts.disabled, ..Extra::new(opts.width) };
        self.field(key, Target::Number { v, spec }, Shape::Number, "", extra)
    }
}
