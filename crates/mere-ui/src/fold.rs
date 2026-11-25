//! `.fold`: a titled section that opens and closes (preview.html lines 113-121).

use mere_layout::{Align, Dim, Edges, Margin, Side, Style};
use mere_base::{Cursor, Vec2, Weight};

use crate::rule::Rule;
use crate::text::TextLook;
use crate::tokens::{GAP, GAP_WIDE, INSET_HAIR};
use crate::ui::{Paint, Resp, Ui};

#[derive(Copy, Clone, Debug, Default)]
pub struct FoldOpts {
    /// `.fold.grow` (line 120): open, it takes the column's free height and so does its body.
    pub grow: bool,
    /// No hairline on top (the page's `border-top`): for a fold that opens a panel with nothing above it to separate
    /// it from.
    pub bare: bool,
}

/// `flex: 1` (grow 1, shrink 1, basis 0) with the automatic minimum switched off.
fn growing(mut s: Style) -> Style {
    s.grow = 1.0;
    s.shrink = 1.0;
    s.basis = Dim::Px(0.0);
    s
}

/// The section's box, a fold's and a section's alike.
pub(crate) fn container_style() -> Style {
    Style { column: true, shrink: 0.0, min_h: Dim::Px(0.0), ..Style::default() }
}

/// The line of the head: `gap` above and below, `gap` between what it holds.
pub(crate) fn head_style(align_items: Align) -> Style {
    Style { pad: Edges::vh(GAP, 0.0), align_items, gap: Vec2::new(GAP, 0.0), ..Style::default() }
}

/// What a section holds: stacked `gap` apart, `inset-hair` under the head and `gap-wide` after.
pub(crate) fn body_style() -> Style {
    Style { column: true, gap: Vec2::new(0.0, GAP), pad: Edges::new(INSET_HAIR, 0.0, GAP_WIDE, 0.0), min_h: Dim::Px(0.0), ..Style::default() }
}

impl Ui {
    pub fn fold(&mut self, key: &str, title: &str, open: &mut bool, body: impl FnOnce(&mut Ui)) -> Resp {
        self.fold_with(key, title, open, FoldOpts::default(), body)
    }

    /// `title` is the app's catalog word ("World"): the widget uppercases it because the page's CSS does
    /// (`text-transform`, line 104). The open state is the app's, as the page keeps its own; a click flips it
    /// before anything is built, so the click's own frame already shows the new state. A closed fold does not
    /// call `body`: what it holds is out of the page (line 112). `Resp.clicked` means `open` flipped.
    pub fn fold_with(&mut self, key: &str, title: &str, open: &mut bool, opts: FoldOpts, body: impl FnOnce(&mut Ui)) -> Resp {
        let t = *self.theme();
        let id = self.key(key);
        let container_style = container_style();
        let container = self.open(key, id, container_style, Paint::None);
        // The border-top: the section rule that opens it.
        if !opts.bare {
            self.rule(Rule::Section, Edges::default());
        }

        let head_id = self.key("head");
        let resp = self.interact(head_id, false, Cursor::Hand);
        if resp.clicked {
            *open = !*open;
        }
        if opts.grow && *open {
            self.restyle(container, growing(container_style));
        }

        self.open("head", head_id, head_style(Align::Baseline), Paint::None);
        self.add_text(&title.to_uppercase(), TextLook::heading(), t.text, false, Style::default());
        // ::after inherits the letter spacing; its weight is 400 (line 116).
        let sign = TextLook { weight: Weight::Regular, ..TextLook::heading() };
        let sign_color = if resp.hovered || resp.held { t.text } else { t.muted };
        let sign_style = Style { margin: Margin { left: Side::Auto, ..Margin::default() }, ..Style::default() };
        self.add_text(if *open { "\u{2212}" } else { "+" }, sign, sign_color, false, sign_style);
        self.close();

        if *open {
            let b = self.key("body");
            let mut style = body_style();
            if opts.grow {
                style = growing(style);
            }
            self.open("body", b, style, Paint::None);
            body(self);
            self.close();
        }
        self.close();
        resp
    }
}
