//! The rule: the one thin line of the kit, `FRAME` thick, as wide as its parent gives it. An atom: every line that
//! parts things (a fold's top, a group's title from its rows, two groups of a menu, a block of a panel from the one
//! above) is this one, so that a change of the line is a change of this file.
//!
//! Two kinds, by what the line parts, each a role of the theme. A section rule (`edge`) opens a section of a panel
//! (a fold, a header, a block); a separator (`line`, a step softer) parts things inside a section (two groups, a
//! group's title from its rows, two rows of figures). The rule at the top of a section is drawn once, by the
//! section, never also by what it holds.

use mere_layout::{Dim, Edges, Margin, Side, Style};

use crate::tokens::FRAME;
use crate::ui::{Paint, Ui};

/// What a rule parts, which sets its colour.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Rule {
    /// Opens a section of a panel: the theme's `edge`.
    Section,
    /// Parts things inside a section: the theme's `line`.
    #[default]
    Separator,
}

impl Ui {
    /// A rule of `kind`, with `around` kept clear on each side (its margin): `Edges::default()` for none.
    pub fn rule(&mut self, kind: Rule, around: Edges) {
        let t = *self.theme();
        let color = match kind {
            Rule::Section => t.edge,
            Rule::Separator => t.line,
        };
        let margin = Margin { left: Side::Px(around.left), top: Side::Px(around.top), right: Side::Px(around.right), bottom: Side::Px(around.bottom) };
        let style = Style { h: Dim::Px(FRAME), shrink: 0.0, margin, ..Style::default() };
        self.add("rule", style, Paint::Box { fill: Some(color), frame: None });
    }
}
