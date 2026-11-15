//! What the bar gives up when the window is too narrow for all of it, worked out from the real widths of its
//! parts before they are built. The widths are arithmetic (the font is monospaced: a word is a count of cells
//! times a constant, see `nui_ui::TextLook::width`), so the thresholds are measured, not typed in: a longer
//! word, another account name or one more view moves them by itself.
//!
//! The bar is a grid of three columns, a free one on each side of the tools so the tools stay centred. A side
//! column is therefore `(width - tools) / 2`, and it has to hold the window's buttons on the right and the
//! brand on the left. The bar gives way in this order, taking the first step that fits:
//!
//! 1. the account name: cut with an ellipsis down to a short stub, then gone whole;
//! 2. the views that are greyed (they need the 16-block detail) move into "More";
//! 3. the word MINECRAFT (the cube stays);
//! 4. the other views move into "More", one at a time from the end of the strip.
//!
//! The pressed view stays in the strip, and the two tools keep their names.

use nui_ui::{TextLook, Tone, tool_button_width};

use crate::text;

/// The window's three buttons, 46 px each: what the right column holds at least.
pub const ENDS_W: f32 = 138.0;
/// `.brand`'s padding before the cube, the cube, and the gap between the brand's parts.
const BRAND_PAD: f32 = 12.0;
const CUBE: f32 = 16.0;
const BRAND_GAP: f32 = 10.0;
/// A strip tab: 13 px each side of its word, overlapping its neighbour's hairline by 1.
const TAB_PAD: f32 = 13.0;
/// The gap between two groups of the strip.
const SEP: f32 = 12.0;
/// The chevron of the "More" tab, and the space before it.
const CHEVRON: f32 = 8.0;
const CHEVRON_GAP: f32 = 7.0;
/// An account cut shorter than this many letters (and the ellipsis) is not worth showing: it goes whole.
pub const ACCOUNT_STUB: usize = 3;

/// What the bar shows, as worked out for one width.
#[derive(Clone, Debug, PartialEq)]
pub struct Form {
    /// The account name is shown (cut with an ellipsis if need be).
    pub account: bool,
    /// The word MINECRAFT is shown.
    pub word: bool,
    /// The views the "More" tab holds, in the strip's order. Empty: no "More" tab.
    pub more: Vec<usize>,
}

impl Default for Form {
    fn default() -> Form {
        Form { account: true, word: true, more: Vec::new() }
    }
}

/// What the plan reads.
#[derive(Clone, Debug)]
pub struct Inputs<'a> {
    /// The window's width.
    pub width: f32,
    /// What the right column holds at least: the window's buttons (`ENDS_W`).
    pub ends: f32,
    pub account: Option<&'a str>,
    /// The preview tool is the one on show (its views take room in the strip).
    pub preview_open: bool,
    /// The pressed view, which always stays in the strip.
    pub pressed: usize,
    /// Which of the six views are greyed.
    pub disabled: [bool; VIEW_COUNT],
    /// 2D and 3D after the views (not built in the section).
    pub shapes: bool,
}

pub const VIEW_COUNT: usize = 6;

/// The word of view `i` (the five views, then the section).
pub fn view_word(i: usize) -> &'static str {
    if i < text::VIEWS.len() { text::VIEWS[i] } else { text::SECTION }
}

fn body() -> TextLook {
    TextLook::body(Tone::Text)
}

/// A strip tab's width less the 1 px it is pulled over its neighbour.
pub fn tab_step(word: &str) -> f32 {
    2.0 * TAB_PAD + body().width(word) - 1.0
}

/// The "More" tab's, the same way.
pub fn more_step() -> f32 {
    2.0 * TAB_PAD + body().width(text::MORE) + CHEVRON_GAP + CHEVRON - 1.0
}

/// The strip's width with the given views moved into "More".
pub fn strip_width(i: &Inputs, more: &[usize]) -> f32 {
    // The strip's own 1 px of padding, then the tools: each pulled 1 px over what is before it.
    let mut w = 1.0 + (tool_button_width(text::TOOL_NAMES[0]) - 1.0) + SEP + (tool_button_width(text::TOOL_NAMES[1]) - 1.0);
    if i.preview_open {
        // The options' box pads its tabs by 1 px on the left, which the first tab's pull takes back.
        w += 1.0;
        w += (0..VIEW_COUNT).filter(|v| !more.contains(v)).map(|v| tab_step(view_word(v))).sum::<f32>();
        if !more.is_empty() {
            w += more_step();
        }
        if i.shapes {
            w += SEP + text::SHAPES.iter().map(|s| tab_step(s)).sum::<f32>();
        }
    }
    w
}

/// The account's narrowest showing: a stub and the ellipsis, or the whole name when that is shorter.
pub fn account_stub_width(account: &str) -> f32 {
    let look = TextLook::body(Tone::Muted);
    look.width(account).min((ACCOUNT_STUB + 1) as f32 * look.width("m"))
}

/// What the brand needs in its column for a form.
pub fn brand_need(i: &Inputs, form: &Form) -> f32 {
    let mut w = BRAND_PAD + CUBE;
    if form.word {
        w += BRAND_GAP + TextLook::heading().width(&text::BAR_TITLE.to_uppercase());
    }
    if let (true, Some(a)) = (form.account, i.account) {
        w += BRAND_GAP + account_stub_width(a);
    }
    w
}

/// Whether the form fits: each side column, `(width - strip) / 2`, holds the window's buttons and the brand.
pub fn fits(i: &Inputs, form: &Form) -> bool {
    let side = (i.width - strip_width(i, &form.more)) / 2.0;
    side + 0.01 >= i.ends.max(brand_need(i, form))
}

/// The first form in the order of giving way that fits, or the last one (nothing left to give) when none does.
pub fn plan(i: &Inputs) -> Form {
    // With no account the "no account" step is the form before it, and costs nothing.
    let mut steps: Vec<Form> = vec![Form::default()];
    if i.account.is_some() {
        steps.push(Form { account: false, ..Form::default() });
    }
    // The greyed views leave first, then the word, then the others from the strip's end. The pressed view stays.
    let greyed: Vec<usize> = (0..VIEW_COUNT).filter(|&v| i.disabled[v] && v != i.pressed).collect();
    let from_end: Vec<usize> = (0..VIEW_COUNT).rev().filter(|&v| !i.disabled[v] && v != i.pressed).collect();
    let held = |k: usize| -> Vec<usize> {
        let mut m = greyed.clone();
        m.extend(from_end.iter().take(k));
        m.sort_unstable();
        m
    };
    let base = Form { account: false, word: true, more: Vec::new() };
    if i.preview_open {
        if !greyed.is_empty() {
            steps.push(Form { more: held(0), ..base.clone() });
        }
        steps.push(Form { word: false, more: held(0), ..base.clone() });
        for k in 1..=from_end.len() {
            steps.push(Form { word: false, more: held(k), ..base.clone() });
        }
    } else {
        steps.push(Form { word: false, ..base });
    }
    let last = steps.len() - 1;
    let mut chosen = steps.into_iter().enumerate().find(|(n, f)| *n == last || fits(i, f)).map(|(_, f)| f).unwrap_or_default();
    // The steps are cumulative, so a form that gave up the word to move views keeps it gone; but the word is only
    // worth giving up when it buys room. The brand is narrower than the window's buttons it shares the side with,
    // so with the real widths it never does, and the word stays unless the brand is the column's widest part.
    if !chosen.word && fits(i, &Form { word: true, ..chosen.clone() }) {
        chosen.word = true;
    }
    chosen
}
